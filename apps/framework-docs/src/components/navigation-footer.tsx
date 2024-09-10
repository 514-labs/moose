import React from "react";
import Link from "next/link";
import { useRouter } from "next/router";
import { ArrowLeft, ArrowRight } from "lucide-react";

interface NavLink {
  title: string;
  href: string;
}

interface CustomNavigationProps {
  prev?: NavLink;
  next?: NavLink;
}

const CustomNavigation: React.FC<CustomNavigationProps> = ({ prev, next }) => {
  const router = useRouter();

  return (
    <div className="custom-navigation">
      {prev && (
        <Link href={prev.href} className="nav-link prev">
          <ArrowLeft className="icon" />
          <span>{prev.title}</span>
        </Link>
      )}
      {next && (
        <Link href={next.href} className="nav-link next">
          <span>{next.title}</span>
          <ArrowRight className="icon" />
        </Link>
      )}
    </div>
  );
};

export default CustomNavigation;
