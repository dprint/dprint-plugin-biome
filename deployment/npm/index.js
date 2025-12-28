const fs = require("fs");
const path = require("path");

/**
 * Gets a buffer representing the Wasm module.
 * @returns {ArrayBuffer}
 */
function getBuffer() {
  return fs.readFileSync(path.join(__dirname, "plugin.wasm"));
}

module.exports = {
  getBuffer,
};
