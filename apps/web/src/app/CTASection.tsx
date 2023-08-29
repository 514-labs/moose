import { Button } from "ui";
import { CTAButton } from "./CTAButton";


const CodeBlockCTA = () => {
  return (
    <div className="flex flex-row">
     <CodeBlock />
     <Button >copy</Button>
    </div>
  )
}

const CodeBlock = () => {
  return (
    <div className="flex flex-row items-center justify-center bg-base-black-250">
      <span className="font-mono py-3 px-6 text-typography-primary "> npx create-igloo-app</span>
    </div>
  )
}

export const CTASection = () => {
  return (
    <div className="bg-black/10 p-5">
      <div className="text-typography-secondary text-2xl my-3">
        start building today
      </div>
      <div className="text-typography-primary my-3">
        igloo is a framework to build data-intensive apps using typescript and sql. it comes with all the typescript primitives you'll need to build a fully featured app that's secure and scales. igloo also provides comes with a CLI to help you be productive while you build your data-intensive application right along side your web app on local machine. no need to configure clusters and networking to start building.
      </div>
      <div>
        <CodeBlockCTA />
      </div>
    </div>
  );
};
