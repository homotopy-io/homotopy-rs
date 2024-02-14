/**
* See a full list of supported triggers at https://firebase.google.com/docs/functions
*/
import { onCall } from "firebase-functions/v2/https";
import { onObjectFinalized } from "firebase-functions/v2/storage";
import * as logger from "firebase-functions/logger";

import { initializeApp } from "firebase-admin/app";
import { getFirestore, Firestore, FieldValue, Timestamp } from "firebase-admin/firestore";
import { getStorage } from "firebase-admin/storage";

initializeApp();

function newUploadNonce(): string {
  return Math.floor(Math.random() * 10000).toString();
}

function newMonthCode(ts: Timestamp): string {
  const date = ts.toDate();
  const year = date.getUTCFullYear().toString().slice(2);
  let month = (date.getUTCMonth() + 1).toString();
  if (month.length === 1) month = `0${month}`;
  return `${year}${month}`;
}

function formatProjectIdx(projectIdx: number): string {
  let idx = projectIdx.toString();
  let zeroes = Math.max(0, 3 - idx.length);
  while (zeroes--) idx = "0" + idx;
  return idx;
}

async function getNewPublishedProjectId(firestore: Firestore): Promise<string> {
  const stateRef = firestore.doc("meta/state");

  const res = await firestore.runTransaction(async (tx) => {
    const stateSnapshot = await tx.get(stateRef);

    const currentMonthCode = newMonthCode(Timestamp.now());
    let projectIdx;

    const state = stateSnapshot.data();
    if (state) {
      projectIdx = state.lastProjectIdx + 1;
      if (currentMonthCode != state.currentMonthCode) projectIdx = 0;
      tx.set(stateRef, {
        lastProjectIdx: projectIdx,
        currentMonthCode,
      });
    } else {
      projectIdx = 0;
      tx.set(stateRef, {
        lastProjectIdx: projectIdx,
        currentMonthCode,
      });
    }
    const formattedProjectIdx = formatProjectIdx(projectIdx);
    return `${currentMonthCode}.${formattedProjectIdx}`;
  });

  return res;
}

export const getUserProjects = onCall(async (req) => {
  if (req.auth) {
    const firestore = getFirestore();
    const uid = (req.data ? req.data.uid : null) || req.auth.uid;
    const args = req.data;

    if (args && args.project && args.project.id) {
      // Return data for a single specified project
      return await getUserProject(firestore, uid, args);
    } else {
      // Return data for all user's projects
      return await getAllUserProjects(firestore, uid, args);
    }
  } else {
    return { status: "failed", message: "Authentication failed" };
  }
});

async function getUserProject(firestore: Firestore, uid: string, args: any): Promise<any> {
  const id = args.project.id;
  logger.info({ args });
  // TODO: auth check

  if (args.project.published) {
    const res = await firestore.runTransaction(async (tx) => {
      const projectSnapshot = await tx.get(firestore.doc(`published-rs/${id}`));
      const projectData = projectSnapshot.data();
      if (projectData) {
        const latestVersion = projectData.latestVersion;

        const versionSnapshot = await tx.get(firestore.doc(`published-rs/${id}/versions/${latestVersion}`));
        const data = versionSnapshot.data();
        if (data) {
          return {
            id,
            uid,
            title: data.title,
            author: data.author,
            abstr: data.abstract,
            visibility: "Published",
            published: true,
            latestVersion: projectData.latestVersion,
            created: projectData.created.seconds,
            lastModified: data.created.seconds,
          };
        }
      }

      return { err: "No project found" };
    });
    return res;
  } else {
    const projectSnapshot = await firestore.doc(`personal-rs/${uid}/projects/${id}`).get();
    const data = projectSnapshot.data();

    if (data) {
      return {
        id,
        uid,
        title: data.title,
        author: data.author,
        abstr: data.abstract,
        visibility: data.visibility,
        published: false,
        created: data.created.seconds,
        lastModified: data.lastModified.seconds,
      };
    }
  }

  return { err: "No project found" };
}

async function getAllUserProjects(firestore: Firestore, uid: string, args: any): Promise<any> {
  const personalSnapshot = await firestore
    .collection(`personal-rs/${uid}/projects`)
    .get();

  const personalProjects = personalSnapshot.docs.map((p) => {
    const data = p.data();
    return {
      id: p.id,
      uid,
      title: data.title,
      author: data.author,
      abstr: data.abstract,
      visibility: data.visibility,
      versionCount: 1,
      created: data.created.seconds,
      lastModified: data.lastModified.seconds,
    };
  });

  const userDoc = await firestore
    .doc(`personal-rs/${uid}`)
    .get();
  let publishedProjects = [];
  const userData = userDoc.data();
  if (userData && userData.published) {
    const publishedProjectData = userData.published.flatMap(async (id: string) => {
      const projectDoc = await firestore
        .doc(`published-rs/${id}`)
        .get();
      const projectData = projectDoc.data();
      if (projectData) {
        const version = projectData.latestVersion;
        const projectVersionDoc = await firestore
          .doc(`published-rs/${id}/versions/${version}`)
          .get();
        const data = projectVersionDoc.data();
        if (data) {
          return {
            id,
            uid,
            title: data.title,
            author: data.author,
            abstr: data.abstract,
            visibility: "Published",
            versionCount: data.latestVersion,
            created: projectData.created.seconds,
            lastModified: projectData.lastModified.seconds,
          };
        }
      }
      return [];
    });
    publishedProjects = await Promise.all(publishedProjectData);
  }

  return {
    personal: personalProjects,
    published: publishedProjects,
  };
}

export const requestBlobUpload = onCall(async (req) => {
  if (req.auth) {
    const firestore = getFirestore();

    const uid = req.auth.uid;
    const args = req.data;

    args.visibility = args.visibility || "Private";

    if (args.visibility === "Published") {
      return await createPublishedProjectVersion(firestore, uid, args);
    }

    const res = await firestore.runTransaction(async (tx) => {
      const projectsRef = firestore.collection(`personal-rs/${uid}/projects`);

      const uploadNonce = newUploadNonce();

      let id = args.id;
      const ts = Timestamp.now();

      const newDoc = {
        created: ts,
        lastModified: ts,
        title: args.title,
        author: args.author,
        abstract: args.abstract,
        visibility: args.visibility,
        uploadNonce,
      };

      if (id) {
        const projectRef = firestore.doc(`personal-rs/${uid}/projects/${id}`);
        tx.update(projectRef, newDoc);
      } else {
        const projectRef = await projectsRef.add(newDoc); // TODO: Data race? (not handled by tx)
        id = projectRef.id;
      }

      return { id, uid, uploadNonce };
    });

    return res;
  } else {
    return { err: "Unauthenticated" };
  }
});

export async function createPublishedProjectVersion(firestore: Firestore, uid: string, args: any): Promise<any> {
  const res = await firestore.runTransaction(async (tx) => {
    const uploadNonce = newUploadNonce();

    const newProject = async (args: any, ts: Timestamp) => {
      const id = await getNewPublishedProjectId(firestore);

      tx.set(firestore.doc(`published-rs/${id}`), {
        created: ts,
        lastModified: ts,
        latestVersion: 0,
        uploadNonce,
      });

      return id;
    };

    let id = args.id;
    let newVersion = 0;
    const ts = Timestamp.now();

    // Get ID for new project version
    if (id) {
      const projectRef = firestore.doc(`published-rs/${id}`);
      const project = await tx.get(projectRef);
      const projectData = project.data();

      if (projectData) {
        newVersion = projectData.latestVersion + 1;
        tx.update(projectRef, {
          lastModified: ts,
          latestVersion: newVersion,
          uploadNonce,
        });
      } else {
        id = await newProject(args, ts);
      }
    } else {
      id = await newProject(args, ts);
    }

    tx.set(firestore.doc(`personal-rs/${uid}`), {
      published: FieldValue.arrayUnion(`${id}`),
    }, { merge: true });

    tx.set(firestore.doc(`published-rs/${id}/versions/${newVersion}`), {
      uid,
      title: args.title,
      author: args.author,
      abstract: args.abstract,
      created: ts,
    });

    return { id, uid, uploadNonce, version: newVersion };
  });
  return res;
}

export const onUpload = onObjectFinalized(async (event: any) => {
  // Get parent folder for filePath
  const filePath = event.data.name;
  const path = filePath.split("/");
  const uid = path[1];
  const id = path[4];
  logger.info(event.data);
  logger.info(`saving ${id} for user ${uid} at ${path}`);

  // Make sure we only look at new 'staged' uploads
  if (path[2] === "upload") {
    if (path[3] === "save") {
      const uploadNonce = path[5];
      return await saveProject(event, uid, id, uploadNonce);
    } else if (path[3] === "publish") {
      const version = path[5];
      const uploadNonce = path[6];
      return await publishProject(event, uid, id, version, uploadNonce);
    }
  }
  return {};
});

async function saveProject(event: any, uid: string, id: string, uploadNonce: string): Promise<any> {
  const firestore = getFirestore();
  const storage = getStorage();

  // TODO: put in transaction?
  // TODO: check uploadNonce
  const docRef = firestore.doc(`personal-rs/${uid}/projects/${id}`);
  const doc = await docRef.get();

  if (doc.exists) {
    logger.info(`moving upload (${id})`);
    await docRef.update({ blob: docRef.path });

    await storage
      .bucket(event.data.bucket)
      .file(event.data.name)
      .move(docRef.path); // TODO: set move opts to avoid race condition

    return { id };
  }
  return { err: "Version invalid" };
}

async function publishProject(event: any, uid: string, id: string, version: string, uploadNonce: string): Promise<any> {
  const firestore = getFirestore();
  const storage = getStorage();

  logger.info({ uid, id, version, uploadNonce });
  // TODO: put in transaction?
  const docRef = firestore.doc(`published-rs/${id}/versions/${version}`);
  const doc = await docRef.get();

  // get next version and update firestore
  if (doc.exists) {
    logger.info(`moving upload ${id}`);
    await docRef.update({ blob: docRef.path });

    await storage
      .bucket(event.data.bucket)
      .file(event.data.name)
      .move(docRef.path); // TODO: set move opts to avoid race condition

    return { id };
  }
  return { err: "Version invalid" };
}

export const updateProjectMetadata = onCall(async (req) => {
  if (req.auth) {
    const firestore = getFirestore();
    const storage = getStorage();

    const uid = req.auth.uid;
    const args = req.data;
    const id = args.id;

    const update: any = {};
    if (args.title !== null) update.title = args.title;
    if (args.author !== null) update.author = args.author;
    if (args.abstract !== null) update.abstract = args.abstract;
    if (args.visibility !== null) update.visibility = args.visibility;

    try {
      const docRef = firestore.doc(`personal-rs/${uid}/projects/${id}`);
      await docRef.update(update);
      const newDoc = await docRef.get();
      const newMetadata = newDoc.data();

      await storage
        .bucket()
        .file(`personal-rs/${uid}/projects/${id}`)
        .setMetadata({
          metadata: {
            HOMOTOPYIO_TITLE: update.title,
            HOMOTOPYIO_AUTHOR: update.author,
            HOMOTOPYIO_ABSTRACT: update.abstract,
            HOMOTOPYIO_VISIBILITY: update.visibility,
          },
        });

      if (newMetadata) {
        return {
          id,
          uid,
          title: newMetadata.title,
          author: newMetadata.author,
          abstr: newMetadata.abstract,
          visibility: newMetadata.visibility,
          published: false,
          versionCount: 1,
          created: newMetadata.created.seconds,
          lastModified: newMetadata.lastModified.seconds,
        };
      } else {
        return { err: "Failed to get updated metadata" };
      }
    } catch (err) {
      return { err: "Update failed" };
    }
  }
  return { err: "Unauthenticated" };
});

export const deleteProject = onCall(async (req) => {
  if (req.auth) {
    const firestore = getFirestore();
    const storage = getStorage();

    const uid = req.auth.uid;
    const args = req.data;
    const id = args.id;

    await firestore
      .doc(`personal-rs/${uid}/projects/${id}`)
      .delete();

    await storage
      .bucket()
      .file(`personal-rs/${uid}/projects/${id}`)
      .delete();
  }
  return { err: "Unauthenticated" };
});
