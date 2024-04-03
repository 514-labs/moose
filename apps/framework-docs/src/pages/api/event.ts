import Mixpanel from "mixpanel";
import type { NextApiRequest, NextApiResponse } from "next";

type ResponseData = {
  message: string;
};

export default function handler(
  req: NextApiRequest,
  res: NextApiResponse<ResponseData>,
) {
  const body = req.body;
  const { name, ...payload } = body;
  const mixpanel = Mixpanel.init("be8ca317356e20c587297d52f93f3f9e");
  mixpanel.track(name, payload);
  res.status(200).json({ message: "Event tracked" });
}
