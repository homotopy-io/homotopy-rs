// This script initializes firebase.

console.log("Loading firebase...");

// Compat modules for firebase ui
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-app-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth-compat.js";

// Import the functions you need from the SDKs you need
import { initializeApp } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-app.js";
import { getAnalytics } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-analytics.js";
import { getPerformance } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-performance.js";

// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
  apiKey: "AIzaSyBbn1EZwUrcwptd56iXezcVTnWeu6I6iac",
  authDomain: "homotopy-test.firebaseapp.com",
  projectId: "homotopy-test",
  storageBucket: "homotopy-test.appspot.com",
  messagingSenderId: "410689461996",
  appId: "1:410689461996:web:5147f0179f66bf2e5bcfc9",
  measurementId: "G-KML50ZHFGS"
};

// Initialize Firebase
const app = firebase.initializeApp(firebaseConfig);
// const analytics = getAnalytics(app);
// const perf = getPerformance(app);
const auth = firebase.auth();

// // Keep track of user log-in state across all pages
// var initApp = function() {
//   firebase.auth().onAuthStateChanged(function(user) {
//     if (user) {
//       // User is signed in.
//       var displayName = user.displayName;
//       var email = user.email;
//       var emailVerified = user.emailVerified;
//       var photoURL = user.photoURL;
//       var uid = user.uid;
//       var phoneNumber = user.phoneNumber;
//       var providerData = user.providerData;
//       user.getIdToken().then(function(accessToken) {
//         //document.getElementById('sign-in-status').textContent = 'Signed in';
//         //document.getElementById('sign-in').textContent = 'Sign out';
//         document.getElementById('account-details').textContent = JSON.stringify({
//           displayName: displayName,
//           email: email,
//           emailVerified: emailVerified,
//           phoneNumber: phoneNumber,
//           photoURL: photoURL,
//           uid: uid,
//           accessToken: accessToken,
//           providerData: providerData
//         }, null, '  ');
//       });
//     } else {
//       // User is signed out.
//       //document.getElementById('sign-in-status').textContent = 'Signed out';
//       //document.getElementById('sign-in').textContent = 'Sign in';
//       document.getElementById('account-details').textContent = 'null';
//     }
//   }, function(error) {
//     console.log(error);
//   });
// };

// window.addEventListener('load', function() {
//   initApp()
// });

