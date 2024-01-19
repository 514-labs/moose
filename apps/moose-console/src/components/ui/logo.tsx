"use client"

import Image from 'next/image'
import { useTheme } from 'next-themes'

function Logo() {
  const { resolvedTheme } = useTheme()
  let src: string 

  switch (resolvedTheme) {
    case 'light':
      src = '/logo-moose-black.svg'
      break
    case 'dark':
      src = '/logo-moose-white.svg'
      break
    default:
      src = '/logo-moose-white.svg'
      break
  }

  return <Image src={src} width={400} height={400} alt='project logo'/>
}

export default Logo