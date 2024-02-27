import Image from "next/image";
import { getData } from "./data";

export default async function Home() {
  const data = await getData();

  return (
    <main className="flex min-h-screen flex-col items-center justify-between p-24">
        {JSON.stringify(data)}
    </main>
  );
}
