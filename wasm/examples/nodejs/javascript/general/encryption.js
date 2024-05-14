const karlsen = require('../../../../nodejs/karlsen');

karlsen.initConsolePanicHook();

(async () => {

    let encrypted = karlsen.encryptXChaCha20Poly1305("my message", "my_password");
    console.log("encrypted:", encrypted);
    let decrypted = karlsen.decryptXChaCha20Poly1305(encrypted, "my_password");
    console.log("decrypted:", decrypted);

})();
