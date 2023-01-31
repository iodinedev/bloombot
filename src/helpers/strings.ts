export const clean = (text: string) => {
  if (typeof text === 'string') {
    return text.replace(/`/g, '\\`').replace(/@/g, '\\@');
  } else {
    return text;
  }
}

export const makeSearchable = (data: string): string => {
  return data.toLowerCase().replace(/[^a-z0-9]/g, '');
};

export const alphanumeric = (str: string) => {
  return str.replace(/[^a-z0-9 ]/gi, '\\$&');
}