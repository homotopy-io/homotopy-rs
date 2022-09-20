// This script initializes firebase.

console.log("Loading firebase...");

// Compat modules for firebase ui
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-app-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-analytics-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-performance-compat.js";
import "https://www.gstatic.com/firebasejs/9.7.0/firebase-auth-compat.js";

// Import the functions you need from the SDKs you need
// import { initializeApp } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-app.js";
// import { getAnalytics } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-analytics.js";
// import { getPerformance } from "https://www.gstatic.com/firebasejs/9.7.0/firebase-performance.js";

// TODO: Add SDKs for Firebase products that you want to use
// https://firebase.google.com/docs/web/setup#available-libraries

// Your web app's Firebase configuration
// For Firebase JS SDK v7.20.0 and later, measurementId is optional
const firebaseConfig = {
  apiKey: "AIzaSyBCtkQM2P7eBAFtKWnlGfyTNyTHE8y5wXY",
  authDomain: "homotopy-io.firebaseapp.com",
  databaseURL: "https://homotopy-io.firebaseio.com",
  projectId: "homotopy-io",
  storageBucket: "homotopy-io.appspot.com",
  messagingSenderId: "872831343483",
  appId: "1:872831343483:web:4de585eb01b14a27ad3bde",
  measurementId: "G-CYGQVSSM6Q"
};

// Initialize Firebase
const app = firebase.initializeApp(firebaseConfig);
const analytics = firebase.analytics();
const perf = firebase.performance();
const auth = firebase.auth();