import { database } from "./database";
import axios from "axios";

const getPage = async (page: number) => {
  console.log(process.env.THINKIFIC_API_KEY)
  const { data } = await axios.get(
    `https://api.thinkific.com/api/public/v1/enrollments`,
    {
      params: {
        page,
        per_page: 100,
        "query[completed]": true,
      },
      headers: {
        "X-Auth-API-Key": process.env.THINKIFIC_API_KEY,
        "X-Auth-Subdomain": "meditation-mind",
        "Content-Type": "application/json",
      }
    }
  );

  console.log(data)

  const clean = data.items.map((item: any) => {
    return item.user_email;
  });

  return { data: clean };
};

export const userCompleted = async (email) => {
  const { data } = await getPage(1);

  if (data.includes(email)) {
    return true;
  }

  const total = Math.ceil(data.total / 100);

  for (let i = 2; i <= total; i++) {
    const { data } = await getPage(i);

    if (data.includes(email)) {
      return true;
    }
  }

  return false;
}