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

export function resgisterAuthCallback(callback) {
  auth.onAuthStateChanged(function(user) {
    if (user) {
      // User is signed in.
      callback(user.displayName);
      // var displayName = user.displayName;
      // var email = user.email;
      // var emailVerified = user.emailVerified;
      // var photoURL = user.photoURL;
      // var uid = user.uid;
      // var phoneNumber = user.phoneNumber;
      // var providerData = user.providerData;
      // user.getIdToken().then(function(accessToken) {
      //   // document.getElementById('account-details').textContent = JSON.stringify({
      //   //   displayName: displayName,
      //   //   email: email,
      //   //   emailVerified: emailVerified,
      //   //   phoneNumber: phoneNumber,
      //   //   photoURL: photoURL,
      //   //   uid: uid,
      //   //   accessToken: accessToken,
      //   //   providerData: providerData
      //   // }, null, '  ');
      //   callback(displayName);
      // });
    } else {
      // User is signed out.
    }
  });
}

export function logOut() {
  auth.signOut();
}
