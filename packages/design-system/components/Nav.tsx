"use client";
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";
import { Disclosure } from "@headlessui/react";
import { Bars3Icon, XMarkIcon } from "@heroicons/react/24/outline";
import Link from "next/link";
import { Text } from "./typography/standard";
import { Grid } from "./containers/page-containers";
import { usePathname } from "next/navigation";
import { cn } from "../lib/utils";

gsap.registerPlugin(SplitText);

const navigation = [
  { name: "docs", href: "https://docs.moosejs.com" },
  { name: "templates", href: "/templates" },
  { name: "blog", href: "https://blog.fiveonefour.com/" },
  { name: "github", href: "https://github.com/514-labs/moose" },
  { name: "community", href: "/community" },
];

export const Nav = () => {
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
            <div className="flex h-16 justify-between items-center">
              <Grid className="sm:grow">
                <div className="col-span-6 flex-shrink-0 grow items-center justify-center text-primary">
                  <Link href="/" className="flex h-full items-center">
                    <Text className="my-0">MooseJS</Text>
                  </Link>
                </div>

                <div className="hidden md:ml-5 col-span-6 md:flex justify-end">
                  {navigation.map((item) => {
                    const isActive = pathname === item.href;

                    return (
                      <div
                        className={cn(
                          isActive
                            ? "items-center text-action-primary px-5"
                            : "items-center text-primary px-5"
                        )}
                        key={item.name}
                      >
                        <Link href={item.href}>
                          <Text
                            className={cn(
                              isActive
                                ? "hover:text-action-primary border-b-2 border-black"
                                : "hover:text-primary",
                              "py-2"
                            )}
                          >
                            {item.name}
                          </Text>
                        </Link>
                        <a></a>
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

          <Disclosure.Panel className="sticky top-0 pt-16 h-screen w-full z-10 bg-background md:hidden">
            <div className="space-y-1 pb-3 pt-2">
              {navigation.map((item) => {
                const isActive = pathname === item.href;

                return (
                  <Disclosure.Button
                    as="a"
                    href={item.href}
                    key={item.name}
                    className={
                      isActive
                        ? "block py-2 pl-10 pr-4 text-5xl text-action-primary  hover:text-primary"
                        : "block py-2 pl-10 pr-4 text-5xl text-primary  hover:text-action-primary"
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
