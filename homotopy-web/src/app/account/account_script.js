// This file is included in `bindings.rs`

import "https://www.gstatic.com/firebasejs/9.7.0/firebase-app-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth-compat.js";

const auth = firebase.auth();
var ui = new firebaseui.auth.AuthUI(auth);

export function initializeUI() {
  //Initialize Firebase UI

  ui.start('#firebaseui-auth-container', {
    signInOptions: [
      // List of OAuth providers supported.
      firebase.auth.GoogleAuthProvider.PROVIDER_ID,
      firebase.auth.GithubAuthProvider.PROVIDER_ID
    ],
    signInFlow: "popup",
    callbacks: {
      signInSuccess: function(currentUser, credential, redirectUrl) {
        return false;
      },
    }
    // Other config options...
  });

}

export function loggedIn() {
  var user = auth.currentUser;
  return (user === null);
}

export function logOut() {
  auth.signOut();
}
