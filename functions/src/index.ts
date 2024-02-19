/**
* See a full list of supported triggers at https://firebase.google.com/docs/functions
*/
import { onCall } from "firebase-functions/v2/https";
import { onObjectFinalized, onObjectDeleted } from "firebase-functions/v2/storage";
import { logger } from "firebase-functions/v2";

import { initializeApp } from "firebase-admin/app";
import { getFirestore, Firestore, FieldValue, Timestamp } from "firebase-admin/firestore";
import { getStorage } from "firebase-admin/storage";

initializeApp();

// return YYMM
function newMonthCode(ts: Timestamp): string {
  const date = ts.toDate();
  const year = date.getUTCFullYear().toString().slice(2);
  let month = (date.getUTCMonth() + 1).toString();
  if (month.length === 1) month = `0${month}`;
  return `${year}${month}`;
}

// return 0-padded identifier
function formatProjectIdx(projectIdx: number): string {
  let idx = projectIdx.toString();
  let zeroes = Math.max(0, 5 - idx.length);
  while (zeroes--) idx = "0" + idx;
  return idx;
}

// e.g. 1501.00001, like arXiv
// https://info.arxiv.org/help/arxiv_identifier.html
async function getNewPublishedProjectId(firestore: Firestore, then: (publishedId: string, tx: FirebaseFirestore.Transaction) => Promise<any>): Promise<string> {
  const stateRef = firestore.doc("meta/state");

  const res = await firestore.runTransaction(async (tx) => {
    const stateSnapshot = await tx.get(stateRef);

    const currentMonthCode = newMonthCode(Timestamp.now());
    let projectIdx;

    const state = stateSnapshot.data();
    if (state) {
      projectIdx = state.lastProjectIdx + 1;
      if (currentMonthCode != state.currentMonthCode) projectIdx = 1;
      tx.set(stateRef, {
        lastProjectIdx: projectIdx,
        currentMonthCode,
      });
    } else {
      projectIdx = 1;
      tx.set(stateRef, {
        lastProjectIdx: projectIdx,
        currentMonthCode,
      });
    }
    const formattedProjectIdx = formatProjectIdx(projectIdx);
    const publishedId = `${currentMonthCode}.${formattedProjectIdx}`;
    await then(publishedId, tx);
    return publishedId;
  });

  return res;
}

export const onCreate = onObjectFinalized(
  { region: "us-east1" },
  async (event: any) => {
    const filePath = event.data.name;
    const path = filePath.split("/");
    if (path[0] === "personal-rs") {
      // path of form "/personal-rs/{uid}/projects/{id}.hom"
      const uid = path[1];
      const id = path[3];
      const metadata = event.data.metadata;
      logger.info(`saving personal project ${id} for user ${uid} at ${filePath}`);
      metadata.public = false;
      metadata.updated = FieldValue.serverTimestamp();
      delete metadata.firebaseStorageDownloadTokens;
      logger.info(`metadata: ${JSON.stringify(metadata)}`);

      // update firestore record with metadata and blob path
      const firestore = getFirestore();
      return firestore.runTransaction(async (tx) => {
        const docRef = firestore.doc(filePath.split(".")[0]);
        tx.set(docRef, { created: FieldValue.serverTimestamp() }, { merge: false });
        tx.update(docRef, metadata, { mergeFields: ["title", "author", "abstract", "updated"] });
        logger.info(`saved personal project ${id} for user ${uid} at ${filePath}`);
      })
    } else if (path[0] === "published-rs") {
      const tag = path[1];
      if (filePath === `published-rs/${tag}/versions/new.hom`) {
        // path of form "/published-rs/{id}/versions/new.hom"
        const metadata = event.data.metadata;

        const firestore = getFirestore();
        return firestore.runTransaction(async (tx) => {
          const docRef = firestore.doc(`published-rs/${tag}`);
          const snapshot = await tx.get(docRef);
          const newVersion = snapshot.get("latest") + 1;
          tx.set(docRef, { latest: newVersion, updated: FieldValue.serverTimestamp() }, { merge: true });
          logger.info(`saving published project ${tag} version ${newVersion}`);
          metadata.created = FieldValue.serverTimestamp();
          delete metadata.firebaseStorageDownloadTokens;
          logger.info(`metadata: ${JSON.stringify(metadata)}`);
          const newDocRef = firestore.doc(`published-rs/${tag}/versions/v${newVersion}`);
          tx.set(newDocRef, metadata, { merge: false });
          const storage = getStorage();
          storage.bucket().file(filePath).rename(`published-rs/${tag}/versions/v${newVersion}.hom`);
          logger.info(`saved published project ${tag} version ${newVersion}`);
        });
      }
    }
  }
);

export const onDelete = onObjectDeleted(
  { region: "us-east1" },
  async (event: any) => {
    const filePath = event.data.name;
    // path of form "/personal-rs/{uid}/projects/{id}"
    const path = filePath.split("/");
    const uid = path[1];
    const id = path[3];
    logger.info(`deleted ${id} for user ${uid} at ${filePath}`);

    // delete firestore record
    const firestore = getFirestore();
    const docRef = firestore.doc(filePath.split(".")[0]);
    return docRef.delete();
  }
);

export const publishPersonal = onCall(
  { region: "us-east1" },
  async (req) => {
    logger.debug(`publishPersonal: ${JSON.stringify(req.data)}`);
    if (req.auth) {
      const uid = req.auth.uid;
      const args = req.data;
      const id = args.id;
      if (!id) {
        logger.error("publishPersonal: No document");
        return { err: "No document" };
      }

      const firestore = getFirestore();
      const storage = getStorage();
      // check blob exists in firestore and storage
      const file = storage.bucket().file(`personal-rs/${uid}/projects/${id}.hom`);
      const docRef = firestore.doc(`personal-rs/${uid}/projects/${id}`);
      const snapshot = await docRef.get();
      if (!file.exists() || !snapshot.exists) {
        logger.error("publishPersonal: Document not found");
        return { err: "Document not found" };
      }
      logger.info(`publishing ${id} for user ${uid}`);
      // get new identifier and move file
      const personalDocRef = firestore.doc(`personal-rs/${uid}`);
      const personalSnapshot = await personalDocRef.get();
      const tag = await getNewPublishedProjectId(firestore, async (tag, tx) => {
        const newDocRef = firestore.doc(`published-rs/${tag}`);
        tx.set(newDocRef, { uid, created: FieldValue.serverTimestamp(), updated: FieldValue.serverTimestamp(), latest: 1 })
        const versionDocRef = firestore.doc(`published-rs/${tag}/versions/v1`);
        tx.set(versionDocRef, { created: FieldValue.serverTimestamp(), title: snapshot.get("title"), author: snapshot.get("author"), abstract: snapshot.get("abstract") })
        const published = (personalSnapshot.get("published") || []);
        published.push(tag);
        tx.set(personalDocRef, { published }, { merge: true })
        file.rename(`published-rs/${tag}/versions/v1.hom`);
      });

      return { tag, version: 1 };
    } else {
      return { err: "Unauthenticated" };
    }
  }
)
