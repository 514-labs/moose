import Image from "next/image";

export const CompareImage = () => (
  <div className="w-full relative py-5">
    <div className=" dark:hidden flex relative aspect-square sm:aspect-video flex-col sm:flex-row">
      <div className="w-full sm:w-[50%] h-full relative">
        <Image
          priority
          src={
            "/images/posts/moose-launch/compare-diagram/compare-std-light.svg"
          }
          fill
          alt={"Moose compare diagram"}
          sizes=" (max-width: 768px) 50vw, 25vw"
        />
      </div>
      <div className="w-full sm:w-[50%] h-full relative">
        <Image
          priority
          src={
            "/images/posts/moose-launch/compare-diagram/compare-data-light.svg"
          }
          fill
          alt={"Moose compare diagram"}
          sizes=" (max-width: 768px) 50vw, 25vw"
        />
      </div>
    </div>
    <div className="hidden dark:flex aspect-square sm:aspect-video flex-col sm:flex-row">
      <div className="w-full sm:w-[50%] h-full relative">
        <Image
          priority
          src={
            "/images/posts/moose-launch/compare-diagram/compare-std-dark.svg"
          }
          fill
          alt={"Moose compare diagram"}
          sizes=" (max-width: 768px) 50vw, 25vw"
        />
      </div>
      <div className="w-full sm:w-[50%] h-full relative">
        <Image
          priority
          src={
            "/images/posts/moose-launch/compare-diagram/compare-data-dark.svg"
          }
          fill
          alt={"Moose compare diagram"}
          sizes=" (max-width: 768px) 50vw, 25vw"
        />
      </div>
    </div>
  </div>
);
