import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { cn } from "@514labs/design-system-components/utils";
import { Text } from "@514labs/design-system-components/typography";
import { Menu, Slash, XIcon } from "lucide-react";
import {
  Sheet,
  SheetContent,
  SheetTrigger,
  SheetHeader,
  SheetClose,
  Button,
  Logo,
} from "@514labs/design-system-components/components";
import { TrackLink } from "@514labs/design-system-components/trackable-components";
import Image from "next/image";

gsap.registerPlugin(SplitText);

interface NavLink {
  eventName: string;
  href: string;
  subject: string;
  content: string;
  emphasized?: boolean;
}

export const FiveOneFourLogo = () => {
  return (
    <TrackLink
      name="logo-link"
      subject="514 home"
      targetUrl="https://www.fiveonefour.com"
      href="https://www.fiveonefour.com"
      className="h-8 w-8 shrink-0"
    >
      <Image
        src="/images/logos/fiveonefour_logo.png"
        alt="logo"
        className="h-8 w-8 rounded-lg"
        priority
        width={32}
        height={32}
      />
    </TrackLink>
  );
};

const NavSlot = ({ children }: { children?: React.ReactNode }) => {
  return (
    <div className="flex flex-row items-center space-x-2">
      <Slash className="w-4 h-4 text-muted-foreground" />

      <div>{children}</div>
    </div>
  );
};

export const MooseLogo = () => {
  return (
    <TrackLink
      name="logo-link"
      subject="moose home"
      targetUrl="https://getmoose.dev"
      href="/"
    >
      <NavSlot>
        <Logo property="Moose" subProperty="JS PY" />
      </NavSlot>
    </TrackLink>
  );
};

const MainNav = () => {
  return (
    <div className="flex flex-row items-center space-x-2">
      <FiveOneFourLogo />
      <MooseLogo />
    </div>
  );
};

const SecondaryNav = ({
  content,
  className,
}: {
  content: NavLink[];
  className?: string;
}) => {
  return (
    <div className={cn("", className)}>
      {content.map((link, index) => {
        return (
          <div
            className={cn("flex items-center text-primary w-full sm:w-auto")}
            key={index}
          >
            <TrackLink
              name={link.eventName}
              subject={link.subject}
              href={link.href}
              targetUrl={link.href}
              className={cn(link.emphasized ? "w-full" : "w-auto")}
            >
              {link.emphasized ? (
                <Button
                  size="lg"
                  className={cn(
                    "py-8 px-4 rounded-xl w-full sm:w-auto sm:text-sm text-lg sm:ml-2 mt-2 sm:my-0",
                    "hover:text-primary-foreground hover:bg-primary/90",
                    "text-primary-foreground",
                  )}
                >
                  {link.content}
                </Button>
              ) : (
                <Text className="py-0 sm:py-2 sm:px-4 my-0">
                  {link.content}
                </Text>
              )}
            </TrackLink>
          </div>
        );
      })}
    </div>
  );
};

const MobileNav = ({ content }: { content: NavLink[] }) => {
  return (
    <Sheet>
      <SheetTrigger asChild>
        <Button size="icon" variant="ghost" className="sm:hidden">
          <Menu />
        </Button>
      </SheetTrigger>
      <SheetContent side="top" className="w-full h-full flex flex-col">
        <SheetHeader className="flex flex-row items-center justify-between">
          <div className="space-x-5 flex flex-row items-center">
            <MainNav />
          </div>
          <SheetClose asChild>
            <Button size="icon" variant="ghost">
              <XIcon />
            </Button>
          </SheetClose>
        </SheetHeader>
        <SecondaryNav
          content={content}
          className="grow mt-0 pt-0 flex flex-col gap-0 text-4xl w-full items-center justify-center"
        />
      </SheetContent>
    </Sheet>
  );
};

export const Nav = () => {
  const mooseNavLinks = [
    {
      content: "Host with Boreal",
      href: "https://boreal.cloud",
      eventName: "nav-link",
      subject: "host with boreal",
    },
    {
      content: "GitHub",
      href: "https://github.com/514-labs/moose",
      eventName: "nav-link",
      subject: "moose github",
    },
    {
      content: "Slack",
      href: "https://join.slack.com/t/moose-community/shared_invite/zt-2345678901-23456789012345678901234567890123",
      eventName: "nav-link",
      subject: "moose slack",
    },
    {
      content: "Blog",
      href: "https://www.fiveonefour.com/blog",
      eventName: "nav-link",
      subject: "moose blog",
    },
    {
      content: "Get Moose",
      href: "https://docs.getmoose.dev",
      eventName: "nav-link",
      subject: "moose docs",
      emphasized: true,
    },
  ];

  return (
    <nav className={cn("sticky top-0 z-50 bg-black p-6 w-full")}>
      <div className="max-w-5xl mx-auto flex items-center justify-between">
        <div className="flex items-center space-x-20">
          <div className="space-x-5 flex flex-row items-center">
            <MainNav />
          </div>
        </div>

        <div>
          <MobileNav content={mooseNavLinks} />
        </div>

        <SecondaryNav
          content={mooseNavLinks}
          className="hidden sm:display items-center sm:flex"
        />
      </div>
    </nav>
  );
};
