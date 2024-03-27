"use client";
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";
import { Disclosure } from "@headlessui/react";
import { Bars3Icon, XMarkIcon } from "@heroicons/react/24/outline";
import { usePathname } from "next/navigation";
import { AnimatedDescription } from "./AnimatedDescription";
import { sendClientEvent } from "../events/sendClientEvent";

gsap.registerPlugin(SplitText);

const navigation = [
  // { name: "Docs", href: "#" },
  //{ name: "about", href: "/about" },
  { name: "docs", href: "https://docs.moosejs.com", target: "_blank" },
  {
    name: "slack",
    href: "https://join.slack.com/t/moose-community/shared_invite/zt-2fjh5n3wz-cnOmM9Xe9DYAgQrNu8xKxg",
    target: "_blank",
  },
  { name: "github", href: "https://github.com/514-labs/", target: "_blank" },
];

interface NavProps {
  identifier: string;
}

export const Nav = ({ identifier }: NavProps) => {
  // const titleRef = React.useRef(null);

  useLayoutEffect(() => {
    // let ctx = gsap.context(() => {
    //   const tl = gsap.timeline();
    //   const splitText = new SplitText(titleRef.current, { type: "words, chars" });
    //   const splitTextChars = splitText.chars;
    //   gsap.set(titleRef.current, { perspective: 400 });
    //   gsap.set(titleRef.current, { visibility: "visible" });
    //   tl.from(splitTextChars,{
    //     y: "-50%",
    //     opacity: 0,
    //     stagger: { each: 0.02 },
    //     });
    // });
    // return () => {
    //   ctx.revert();
    // }
  }, []);
  const pathname = usePathname();

  return (
    <Disclosure as="nav" className="fixed top-0 w-full z-50">
      {({ open }) => (
        <>
          <div className="px-8 z-50 sticky w-full bg-black backdrop-blur-2xl bg-gray-200/80 lg:px-10 py-2">
            <div className="flex h-16 justify-between">
              <div className="flex flex-grow">
                <div className="flex flex-shrink-0 grow items-center text-white">
                  <a href="/">
                    <AnimatedDescription
                      position={0.2}
                      className="px-0 w-full"
                      content="moosejs"
                    />
                  </a>
                </div>

                <div className="hidden sm:ml-6 sm:flex sm:flex-grow">
                  {/* Current: "border-action-primary text-gray-900", Default: "border-transparent text-gray-500  hover:text-action-primary" */}
                  {navigation.map((item) => {
                    const isActive = pathname === item.href;

                    return (
                      <div
                        className={
                          isActive
                            ? "flex-grow flex items-center justify-end text-action-primary "
                            : "flex-grow flex items-center justify-end text-black"
                        }
                        key={item.name}
                      >
                        <a
                          onClick={async () => {
                            await sendClientEvent("nav-click", identifier, {
                              href: item.href,
                            });
                          }}
                          href={item.href}
                          className={
                            isActive
                              ? "hover:text-action-primary"
                              : "hover:text-white"
                          }
                        >
                          <AnimatedDescription
                            position={0.3}
                            className="px-0 w-full"
                            content={item.name}
                          />
                        </a>
                      </div>
                    );
                  })}
                </div>
              </div>
              <div className="-mr-2 flex items-center sm:hidden">
                {/* Mobile menu button */}
                <Disclosure.Button className="relative inline-flex items-center justify-center rounded-md p-2 text-black focus:outline-none focus:ring-2 focus:ring-inset focus:ring-black">
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

          <Disclosure.Panel className="sticky top-0 pt-16 h-screen w-full z-10 bg-gray-200 sm:hidden">
            <div className="space-y-1 pb-3 pt-2">
              {/* Current: "border-action-primary text-action-primary", Default: "border-transparent text-white hover:bg-gray-50  hover:text-action-primary" */}
              {navigation.map((item) => {
                const isActive = pathname === item.href;

                return (
                  <Disclosure.Button
                    as="a"
                    href={item.href}
                    key={item.name}
                    className={
                      isActive
                        ? "block py-2 pl-10 pr-4 text-5xl text-action-primary hover:text-white"
                        : "block py-2 pl-10 pr-4 text-5xl hover:text-action-primary"
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
