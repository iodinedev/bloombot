import { removeMessage } from '../helpers/starboard'
import { rollbar } from '../helpers/rollbar'

export = async (client, message, channel) => {
  try {
    await removeMessage(client, message)
  } catch (err: any) {
    rollbar.error(err);
  }
}
