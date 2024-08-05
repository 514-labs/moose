"use client";
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import { Disclosure } from "@headlessui/react";
import { Bars3Icon, XMarkIcon } from "@heroicons/react/24/outline";
import { Text } from "./typography/standard";
import { Grid } from "./containers/page-containers";
import { usePathname } from "next/navigation";
import { cn } from "../lib/utils";
import { Button } from "./ui/button";
import { Logo } from "./logo";
import { Badge } from "./ui/badge";
import { TrackLink } from "./trackable-components";
import { ProductBadge } from "./language-badge";
import {
  NavigationMenu,
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuTrigger,
  NavigationMenuViewport,
} from "./ui/navigation-menu";

gsap.registerPlugin(SplitText);

interface NavProps {
  property: string;
  subProperty?: string;
  navigation: { name: string; href: string; emphasized?: boolean }[];
}

export const Nav = ({ property, subProperty, navigation }: NavProps) => {
  useLayoutEffect(() => {}, []);
  const pathname = usePathname();

  return (
    <Disclosure
      as="nav"
      className="sticky top-0 w-full z-50 px-5 bg-background"
    >
      {({ open }) => (
        <>
          <div className="z-50 sticky w-full py-2">
            <div className="flex h-20 justify-between items-center">
              <Grid className="grow md:grid md:grid-cols-12 md:gap-x-10">
                <div className="col-span-6 flex-shrink-0 grow items-center justify-center text-primary content-center">
                  <div className="flex flex-row items-center content-center justify-between w-fit p-10">
                    <TrackLink
                      name={"Nav"}
                      subject="home"
                      href="/"
                      className="flex items-center"
                    >
                      <Logo property={property} subProperty={subProperty} />
                    </TrackLink>
                    {property !== "fiveonefour" ? (
                      <TrackLink
                        name={"Link"}
                        subject="Language Badge"
                        href={"https://docs.moosejs.com"}
                      >
                        <div className="flex flex-row items-center content-center">
                          <ProductBadge name="" tag="JS" />
                          <ProductBadge name="" tag="PY" />
                        </div>
                      </TrackLink>
                    ) : null}
                  </div>
                </div>

                <div className="hidden md:ml-5 col-span-6 md:flex justify-end">
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
              </Grid>
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
            </div>
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
        </>
      )}
    </Disclosure>
  );
};
