import { starboardActions } from '../helpers/starboard';

export = async (client, message, channel) => {
  starboardActions.removeMessage(client, message);
};
