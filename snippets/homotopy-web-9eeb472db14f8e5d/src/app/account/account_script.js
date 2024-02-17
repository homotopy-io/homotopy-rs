// This file is included in `bindings.rs`

const getUserProjectsCF = functions.httpsCallable('getUserProjects');
const requestBlobUploadCF = functions.httpsCallable('requestBlobUpload');
const updateProjectMetadataCF = functions.httpsCallable('updateProjectMetadata');
const deleteProjectCF = functions.httpsCallable('deleteProject');

export function initUI() {
  // Initialise Firebase UI
  ui.start('#firebaseui-auth-container', {
    signInOptions: [
      // List of OAuth providers supported.
      firebase.auth.GoogleAuthProvider.PROVIDER_ID,
      firebase.auth.GithubAuthProvider.PROVIDER_ID
    ],
    signInFlow: "popup",
    callbacks: {
      signInSuccessWithAuthResult: function (authResult, redirectUrl) {
        return false;
      },
    }
    // Other config options...
  });
}

export function registerAuthCallback(logInCallback, unsubscribe) {
  if (unsubscribe) unsubscribe();

  return auth.onAuthStateChanged(
    (user) => {
      if (user) {
        // User is signed in.
        console.log({ displayName: user.displayName, photoURL: user.photoURL });
        logInCallback(user.displayName, user.photoURL);

        // Other useful data
        // var displayName = user.displayName;
        // var email = user.email;
        // var emailVerified = user.emailVerified;
        // var uid = user.uid;
        // var phoneNumber = user.phoneNumber;
        // var providerData = user.providerData;
      } else {
        // User is signed out.
      }
    }, (error) => {
      console.error(error);
    });
}

export function logOut(logOutCallback) {
  auth.signOut()
    .then(() => {
      logOutCallback();
    })
    .catch((error) => {
      console.error(error);
    });
}

export async function getUserProjects(maybeProject, projectsCallback) {
  if (auth.currentUser) {
    // User is signed in
    try {
      const res = await getUserProjectsCF(maybeProject)
      return projectsCallback(res.data);
    } catch (err) {
      console.error(err);
    }
  }
}

export async function saveProject(args, saveCallback) {
  if (auth.currentUser) {
    const uid = auth.currentUser.uid;
    console.log(uid);
    let id = args.id();
    console.log(id);

    const requestBlobUpload = await requestBlobUploadCF({
      id,
      title: args.title(),
      author: args.author(),
      abstract: args.abstr(),
      visibility: args.visibility(),
    });
    console.log(requestBlobUpload);
    if (!requestBlobUpload.data) return saveCallback();

    const uploadNonce = requestBlobUpload.data.uploadNonce;
    const version = requestBlobUpload.data.version;
    id = requestBlobUpload.data.id;

    const filePath = args.visibility() === "Published"
      ? `personal-rs/${uid}/upload/publish/${id}/${version}/${uploadNonce}`
      : `personal-rs/${uid}/upload/save/${id}/${uploadNonce}`;

    // Upload blob where filename is ID
    const fileRef = storage.ref().child(filePath);

    const metadata = {
      customMetadata: {
        HOMOTOPYIO_TITLE: args.title(),
        HOMOTOPYIO_AUTHOR: args.author(),
        HOMOTOPYIO_ABSTRACT: args.abstr(),
        HOMOTOPYIO_VISIBILITY: args.visibility(),
        HOMOTOPYIO_UPLOAD_NONCE: uploadNonce,
      }
    };

    // Here, we should check if the file already exists on the server.
    try {
      await fileRef.put(args.blob(), metadata);
    } catch (err) {
      console.error(err);
      return saveCallback();
    }
    console.log("uploaded blob");

    return saveCallback({
      id,
      uid,
      title: args.title(),
      author: args.author(),
      abstr: args.abstr(),
      visibility: args.visibility(),
      published: false,
      created: 0,
      lastModified: 0,
    });
  }
  return saveCallback();
}

export async function updateProjectMetadata(args, updateCallback) {
  if (auth.currentUser) {
    const uid = auth.currentUser.uid;
    const id = args.id();

    console.log({ visibility: args.visibility() });
    try {
      const res = await updateProjectMetadataCF({
        id,
        title: args.title(),
        author: args.author(),
        abstract: args.abstract(),
        visibility: args.visibility(),
      });
      if (res.data.err) throw new Error(res.data.err);
      return updateCallback(res.data);
    } catch (err) {
      console.error(err);
      return updateCallback();
    }
  }
  return updateCallback();
}

export async function deleteProject(args, deleteCallback) {
  if (auth.currentUser) {
    const uid = auth.currentUser.uid;
    const id = args.id();

    try {
      await deleteProjectCF({ id });
      return deleteCallback(id);
    } catch (err) {
      console.error(err);
      return deleteCallback();
    }
  }
  return deleteCallback();
}

export async function downloadProject(args, downloadCallback) {
  const uid = args.uid() || (auth.currentUser ? auth.currentUser.uid : null);
  const id = args.id();
  const published = args.published();
  const specificVersion = args.specificVersion();

  let fileRef;
  const storageRef = storage.ref();

  console.log({ id, published, specificVersion });

  const project = await getUserProjectsCF({
    project: { id, published, specificVersion },
  });
  if (published) {
    if (specificVersion) {
      fileRef = storageRef.child(`published-rs/${id}/versions/${specificVersion}`);
    } else {
      console.log({ project });
      if (project.data) {
        const latestVersion = project.data.latestVersion;
        fileRef = storageRef.child(`published-rs/${id}/versions/${latestVersion}`);
      }
    }
  } else {
    fileRef = storageRef.child(`personal-rs/${uid}/projects/${id}`);
  }

  if (!fileRef) {
    console.error(`Can't find project with id ${id}`);
    return downloadCallback();
  }

  try {
    const downloadUrl = await fileRef.getDownloadURL();
    const data = await fetch(downloadUrl);
    const blob = await data.arrayBuffer();
    console.log(blob);
    console.log(project.data);
    return downloadCallback([project.data, new Uint8Array(blob)]);
  } catch (err) {
    console.error(err);
    return downloadCallback();
  }
  return downloadCallback();
}
