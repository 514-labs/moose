"use client";
import { useSession, signIn } from "next-auth/react";
import DashboardPage from "./dashboard_page";

export default function Home() {
  const { data: session } = useSession();
  if (!session) {
    return (
      <>
        <p>Not signed in</p>
        <br />
        <button onClick={() => signIn()}>Sign in</button>
      </>
    );
  }

  return <DashboardPage />;
}
