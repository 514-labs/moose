'use client'
import { useLayoutEffect } from "react";
import { gsap } from "gsap";
import { SplitText } from "gsap/SplitText";
import React from "react";
import { Fragment } from 'react'
import { Disclosure, Menu, Transition } from '@headlessui/react'
import { Bars3Icon, BellIcon, XMarkIcon } from '@heroicons/react/24/outline'

gsap.registerPlugin(SplitText);


function classNames(...classes) {
  return classes.filter(Boolean).join(' ')
}

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
    <Disclosure as="nav" className="backdrop-blur-sm bg-black/60 fixed w-full z-50">
      {({ open }) => (
        <>
          <div className="px-4 sm:px-6 lg:px-8">
            <div className="flex h-16 justify-between">
              <div className="flex flex-grow">
                <div className="flex flex-shrink-0 grow items-center text-white">
                  igloo
                </div>
                <div className="hidden sm:ml-6 sm:flex sm:flex-grow">
                  {/* Current: "border-action-primary text-gray-900", Default: "border-transparent text-gray-500 hover:border-gray-300 hover:text-action-primary" */}
                  <a
                    href="#"
                    className="flex grow items-center justify-end text-white hover:border-gray-300 hover:text-action-primary"
                  >
                    Docs
                  </a>
                  <a
                    href="#"
                    className="flex grow items-center justify-end text-white hover:border-gray-300 hover:text-action-primary"
                  >
                    Github
                  </a>
                  <a
                    href="#"
                    className="flex grow items-center justify-end text-white hover:border-gray-300 hover:text-action-primary"
                  >
                    Discord
                  </a>
                  <a
                    href="#"
                    className="flex grow items-center justify-end text-white hover:border-gray-300 hover:text-action-primary"
                  >
                    Join us
                  </a>
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

          <Disclosure.Panel className="sm:hidden">
            <div className="space-y-1 pb-3 pt-2">
              {/* Current: "border-action-primary text-action-primary", Default: "border-transparent text-white hover:bg-gray-50 hover:border-gray-300 hover:text-action-primary" */}
              <Disclosure.Button
                as="a"
                href="#"
                className="block border-l-4 border-transparent py-2 pl-3 pr-4 text-base text-white hover:border-gray-300 hover:bg-gray-50 hover:text-action-primary"
              >
                Docs
              </Disclosure.Button>
              <Disclosure.Button
                as="a"
                href="#"
                className="block border-l-4 border-action-primary py-2 pl-3 pr-4 text-base text-action-primary"
              >
                Github
              </Disclosure.Button>
              <Disclosure.Button
                as="a"
                href="#"
                className="block border-l-4 border-transparent py-2 pl-3 pr-4 text-base text-white hover:border-gray-300 hover:bg-gray-50 hover:text-action-primary"
              >
                Discord
              </Disclosure.Button>
              <Disclosure.Button
                as="a"
                href="#"
                className="block border-l-4 border-transparent py-2 pl-3 pr-4 text-base text-white hover:border-gray-300 hover:bg-gray-50 hover:text-action-primary"
              >
                Join us
              </Disclosure.Button>
            </div>
          </Disclosure.Panel>
        </>
      )}
    </Disclosure>
  );
};






