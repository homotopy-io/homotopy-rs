// This file is included in `bindings.rs`

export function getFirebase() {
  return app.firebase;
}

export function initUI(onSignInSuccessWithAuthResult) {
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
        onSignInSuccessWithAuthResult(authResult);
        return false; // don't continue to redirect automatically
      },
    }
  });
}

export async function publishPersonal(id) {
  const f = functions.httpsCallable('publishPersonal');
  return f({ id });
}
