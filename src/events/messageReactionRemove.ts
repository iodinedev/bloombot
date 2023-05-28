import { removeStar } from '../helpers/starboard'
import { rollbar } from '../helpers/rollbar'

export = async (client, reaction, user) => {
  try {
    // When we receive a reaction we check if the reaction is partial or not, return because raw should fire this event
    if (!reaction || reaction.partial) {
      return
    }

    await removeStar(client, user, reaction)
  } catch (err: any) {
    rollbar.error(err);
  }
}
