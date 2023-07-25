import Rollbar from "rollbar";

var rollbar;

if (process.env.TEST_GUILD_ID) {
  console.log("Rollbar is disabled because this is a test environment.")

  rollbar = {
    error: (err: any) => console.error(err),
    warn: (err: any) => console.warn(err),
    info: (err: any) => console.info(err),
    debug: (err: any) => console.debug(err),
    critical: (err: any) => console.error(err),
    log: (err: any) => console.log(err),
  }
} else {
  rollbar = new Rollbar({
    accessToken: process.env.ROLLBAR_ACCESS_TOKEN,
    captureUncaught: true,
    captureUnhandledRejections: true,
  });
}

export { rollbar };