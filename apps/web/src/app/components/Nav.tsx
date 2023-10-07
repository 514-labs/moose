'use client'
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";
import { Disclosure } from '@headlessui/react'
import { Bars3Icon,  XMarkIcon } from '@heroicons/react/24/outline'

gsap.registerPlugin(SplitText);

const navigation = [
  // { name: "Docs", href: "#" },
  { name: "Github", href: "https://github.com/514-labs/igloo-stack" },
  { name: "Discord", href: "https://discord.gg/WX3V3K4QCc" },
  // { name: "Join us", href: "#" },
]

export const Nav = () => {
  const titleRef = React.useRef(null);

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

  return (
    <Disclosure as="nav" className="fixed w-full z-50">
      {({ open }) => (
        <>
          <div className="px-4 z-50 fixed w-full bg-black sm:backdrop-blur-md sm:bg-black/60 sm:px-6 lg:px-8 ">
            <div className="flex h-16 justify-between">
              <div className="flex flex-grow">
                <div className="flex flex-shrink-0 grow items-center text-white">
                  <a  href="/">
                    igloo
                  </a>
                </div>
                
                <div className="hidden sm:ml-6 sm:flex sm:flex-grow">
                  {/* Current: "border-action-primary text-gray-900", Default: "border-transparent text-gray-500  hover:text-action-primary" */}
                  {
                    navigation.map((item) => (
                      <div className="flex-grow flex items-center justify-end text-white  " key={item.name}>
                      <a
                        href={item.href}
                        className="hover:text-action-primary" 
                      >
                        {item.name}
                      </a>
                      </div>
                    ))
                  }
                </div>
              </div>
              <div className="-mr-2 flex items-center sm:hidden">
                {/* Mobile menu button */}
                <Disclosure.Button className="relative inline-flex items-center justify-center rounded-md p-2 text-white hover:text-action-primary focus:outline-none focus:ring-2 focus:ring-inset focus:ring-action-primary">
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

          <Disclosure.Panel className="fixed top-0 pt-16 h-screen w-full z-10 bg-black sm:hidden">
            <div className="space-y-1 pb-3 pt-2">
              {/* Current: "border-action-primary text-action-primary", Default: "border-transparent text-white hover:bg-gray-50  hover:text-action-primary" */}
              {
                navigation.map((item) => (
                  <Disclosure.Button
                    as="a"
                    href={item.href}
                    key={item.name}
                    className="block py-2 pl-10 pr-4 text-5xl text-white  hover:text-action-primary"
                  >
                    {item.name}
                  </Disclosure.Button>
                ))
              }
            </div>
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  );
};






