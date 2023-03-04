import { removeMessage } from '../helpers/starboard'

export = async (client, message, channel) => {
  await removeMessage(client, message)
}
