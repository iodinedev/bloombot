import Rollbar from "rollbar";

var rollbar;

if (process.env.TEST_GUILD_ID) {
  console.info("[INFO] Rollbar is disabled because this is a test environment.")

  rollbar = {
    error: (err: any) => console.error("[ERROR] " + err),
    warn: (err: any) => console.warn("[WARNING] " + err),
    info: (err: any) => console.info("[INFO] " + err),
    debug: (err: any) => console.debug("[DEBUG] " + err),
    critical: (err: any) => console.error("[CRITICAL] " + err),
    log: (err: any) => console.log("[LOG] " + err),
  }
} else {
  rollbar = new Rollbar({
    accessToken: process.env.ROLLBAR_ACCESS_TOKEN,
    captureUncaught: true,
    captureUnhandledRejections: true,
  });
}

export { rollbar };