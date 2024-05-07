import { withNextVideo } from "next-video/process";
/** @type {import('next').NextConfig} */
const nextConfig = {
  transpilePackages: ["design-system"],
};

export default withNextVideo(nextConfig);
