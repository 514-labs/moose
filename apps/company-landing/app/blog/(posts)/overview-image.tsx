import Image from "next/image";

export const OverviewImage = () => (
  <div className="w-full relative aspect-video">
    <Image
      priority
      className="hidden dark:block"
      src={"/images/posts/moose-launch/full-dark-diagram.svg"}
      fill
      alt={"Moose overview diagram"}
      sizes=" (max-width: 768px) 50vw, 25vw"
    />

    <Image
      priority
      className="block dark:hidden"
      src={"/images/posts/moose-launch/full-light-diagram.svg"}
      fill
      alt={"Moose overview diagram"}
      sizes=" (max-width: 768px) 50vw, 25vw"
    />
  </div>
);
