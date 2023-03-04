import { checkReport } from '../helpers/report'
import { addStar } from '../helpers/starboard'

export = async (client, reaction, user) => {
  // When we receive a reaction we check if the reaction is partial or not, and return because this event will be fired by raw
  if (!reaction || reaction.partial) {
    return
  }

  // Check if user is reporting a message
  await checkReport(client, user, reaction)
  // Check if message should be added to starboard or if starboard message should be updated
  await addStar(client, user, reaction)
}
