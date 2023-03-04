import { removeStar } from '../helpers/starboard'

export = async (client, reaction, user) => {
  // When we receive a reaction we check if the reaction is partial or not, return because raw should fire this event
  if (reaction === null || reaction.partial !== null) {
    return
  }

  await removeStar(client, user, reaction)
}
