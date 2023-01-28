export const makeSearchable = (data: any) => {
  return data.toLowerCase().replace(/[^a-z0-9]/g, '');
};