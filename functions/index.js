// Create and Deploy Your First Cloud Functions
// https://firebase.google.com/docs/functions/write-firebase-functions

// Initialize
const functions = require("firebase-functions");
const admin = require("firebase-admin");
admin.initializeApp();

/* 
------------------------------
	USER AUTHENTICATION
------------------------------
*/

// When a new user is created, create a database entry for the user
exports.newUserSignup = functions.auth.user().onCreate(user => {
	return admin.firestore().collection('users').doc(user.uid).set({
		projects: [],
	});
});

// When a user is deleted, delete its database entry
exports.newDeleted = functions.auth.user().onDelete(user => {
	const doc = admin.firestore().collection('users').doc(user.uid);
	return doc.delete();
});

