const functions = require('firebase-functions');
const app = require('../dist/homotopy_server');

exports.api = functions.region('europe-west2').https.onRequest(async (request, response) => {
  try {
    let res = await app.handle_request(request.url);
    functions.logger.info(res);
    response.send(res);
  } catch (e) {
    functions.logger.error(e);
  }
});
