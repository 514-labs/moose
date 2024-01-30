import { Metadata } from "next";

export const metadata: Metadata = {
  title: "MooseJS | Build for the modern data stack",
  openGraph: {
    images: "/open-graph/og_igloo_4x.webp",
  },
};



export default async function Infrastructure(): Promise<JSX.Element> {

  return (
    <div>hello</div>
  );
}