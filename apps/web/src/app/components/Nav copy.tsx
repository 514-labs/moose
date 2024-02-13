'use client'
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";
import { Disclosure } from '@headlessui/react'
import { Bars3Icon,  XMarkIcon } from '@heroicons/react/24/outline'
import { usePathname } from 'next/navigation'
import { AnimatedDescription } from "./AnimatedDescription";

gsap.registerPlugin(SplitText);

const navigation = [
  // { name: "Docs", href: "#" },
  { name: "about", href: "/about" },
  { name: "careers", href: "/careers" },
  { name: "community", href: "/community" },
  // { name: "Join us", href: "#" },
]

export const Nav = () => {
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
  const pathname = usePathname()

  return (
    <Disclosure as="nav" className="sticky top-0 w-full z-50">
      {({ open }) => (
        <>
          <div className="px-8 z-50 sticky w-full bg-black backdrop-blur-2xl bg-black/80 lg:px-10 py-2">
            <div className="flex h-16 justify-between">
              <div className="flex flex-grow">
                <div className="flex flex-shrink-0 grow items-center text-white">
                  <a href="/">
                  <AnimatedDescription position={1} className="px-0 w-full" content="fiveonefour"/>
                  </a>
                </div>
                
                <div className="hidden sm:ml-6 sm:flex sm:flex-grow">
                  {/* Current: "border-action-primary text-gray-900", Default: "border-transparent text-gray-500  hover:text-action-primary" */}
                  {
                    navigation.map((item) => {
                      const isActive = pathname === item.href;
                      
                      return (
                        <div className={isActive ? "flex-grow flex items-center justify-end text-action-primary " : "flex-grow flex items-center justify-end text-white "} key={item.name}>
                          <a
                            href={item.href}
                            className={ isActive ? "hover:text-action-primary" : "hover:text-white" }
                          >
                            <AnimatedDescription position={1} className="px-0 w-full" content={item.name}/>
                          </a>
                        </div>
                    )})
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

          <Disclosure.Panel className="sticky top-0 pt-16 h-screen w-full z-10 bg-black sm:hidden">
            <div className="space-y-1 pb-3 pt-2">
              {/* Current: "border-action-primary text-action-primary", Default: "border-transparent text-white hover:bg-gray-50  hover:text-action-primary" */}
              {
                navigation.map((item) => {
                  const isActive = pathname === item.href;

                  return (
                  <Disclosure.Button
                    as="a"
                    href={item.href}
                    key={item.name}
                    className={
                      isActive ? 
                      "block py-2 pl-10 pr-4 text-5xl text-action-primary  hover:text-white" : 
                      "block py-2 pl-10 pr-4 text-5xl text-white  hover:text-action-primary" 
                    }
                  >
                    {item.name}
                  </Disclosure.Button>
                )})
              }
            </div>
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  );
};






