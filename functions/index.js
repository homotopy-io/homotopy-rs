// Initialize
const functions = require("firebase-functions");
const admin = require("firebase-admin");
admin.initializeApp();

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

exports.saveProject = functions.https.onCall((data, context) => {
    const doc = admin.firestore().collection('users').doc(data.uid);
    return doc.get().then(snapshot => {
        let projects = {};
        if (!snapshot.exists) {
            projects[data.project.id] = data.project.data;
        } else {
            projects = snapshot.get("projects");
            projects[data.project.id] = data.project.data;
        }
        doc.set({
            projects,
        });
    });
});

exports.getUserProjects = functions.https.onCall((uid, context) => {
    const doc = admin.firestore().collection('users').doc(uid);
    return doc.get().then(doc => {
        let projects = [];
        if (doc.exists) {
            projects = Object.entries(doc.data().projects).map(([id, data]) => {return {id, data};});
        }
        return { projects };
    });
});
