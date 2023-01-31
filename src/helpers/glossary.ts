export const makeSearchable = (data: string): string => {
  return data.toLowerCase().replace(/[^a-z0-9]/g, '');
};