"use client";
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { Disclosure } from "@headlessui/react";
import { Bars3Icon, XMarkIcon } from "@heroicons/react/24/outline";
import { usePathname } from "next/navigation";
import { cn } from "@514labs/design-system-components/utils";
import { Text } from "@514labs/design-system-components/typography";
import { Button, Logo } from "@514labs/design-system-components/components";
import { TrackLink } from "./trackable-components";
import { Slash } from "lucide-react";

import Image from "next/image";

gsap.registerPlugin(SplitText);

interface NavProps {
  property: string;
  navigation: { name: string; href: string; emphasized?: boolean }[];
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

export const Nav = ({ navigation }: NavProps) => {
  useLayoutEffect(() => {}, []);
  const pathname = usePathname();

  return (
    <Disclosure
      as="nav"
      className="sticky top-0 z-50 px-5 py-5 lg:py-0 bg-black-100/90 backdrop-blur-2xl w-full"
    >
      {({ open }) => (
        <>
          <div className="max-w-5xl mx-auto flex flex-row items-center justify-between">
            {/* Logo */}
            <div className="flex items-center md:space-x-20">
              <div className="space-x-5 flex flex-row items-center">
                <div className="flex flex-row items-center space-x-2">
                  <FiveOneFourLogo />
                  <MooseLogo />
                </div>
              </div>
            </div>
            {/* Desktop Nav Items */}
            <div className="justify-end hidden sm:display items-center sm:flex">
              {navigation.map((item) => {
                const isActive = pathname.startsWith(item.href);
                return (
                  <div
                    className={cn(
                      isActive
                        ? "flex items-center text-action-primary"
                        : "flex items-center text-primary",
                    )}
                    key={item.name}
                  >
                    <TrackLink
                      name={"Nav Click"}
                      subject={item.name}
                      href={item.href}
                    >
                      {item.emphasized ? (
                        <Button size={"lg"} className="py-8">
                          <Text
                            className={cn(
                              isActive
                                ? "hover:text-action-primary-foreground "
                                : "hover:text-primary-foreground",
                              "text-primary-foreground",
                            )}
                          >
                            {item.name}
                          </Text>
                        </Button>
                      ) : (
                        <Text
                          className={cn(
                            isActive
                              ? "hover:text-action-primary border-b-2 border-primary"
                              : "hover:text-primary border-b-2 border-transparent",
                            "py-2 px-5",
                          )}
                        >
                          {item.name}
                        </Text>
                      )}
                    </TrackLink>
                  </div>
                );
              })}
            </div>
            <div className="-mr-2 flex items-center md:hidden">
              {/* Mobile menu button */}
              <Disclosure.Button className="relative inline-flex items-center justify-center rounded-md p-2 text-primary hover:text-action-primary focus:outline-none focus:ring-2 focus:ring-inset focus:ring-action-primary">
                <span className="absolute -inset-0.5" />
                <span className="sr-only">Open main menu</span>
                {open ? (
                  <XMarkIcon className="block h-6 w-6" aria-hidden="true" />
                ) : (
                  <Bars3Icon className="block h-6 w-6" aria-hidden="true" />
                )}
              </Disclosure.Button>
            </div>

            <Disclosure.Panel className="sticky top-20 h-screen md:h-auto justify-center w-full z-10 bg-background md:hidden">
              <div className="space-y-1 pb-3 pt-2 mt-[25%]">
                {navigation.map((item) => {
                  const isActive = pathname.startsWith(item.href);

                  return (
                    <Disclosure.Button
                      as="a"
                      href={item.href}
                      key={item.name}
                      className={
                        isActive
                          ? "block py-2 pl-0 pr-4 text-5xl text-action-primary hover:text-primary"
                          : "block py-2 pl-0 pr-4 text-5xl text-primary hover:text-action-primary"
                      }
                    >
                      {item.name}
                    </Disclosure.Button>
                  );
                })}
              </div>
            </Disclosure.Panel>
          </div>
        </>
      )}
    </Disclosure>
  );
};
