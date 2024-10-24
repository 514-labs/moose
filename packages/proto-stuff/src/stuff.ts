import { InfrastructureMap } from "./gen/infrastructure_map";

const asdf: InfrastructureMap = InfrastructureMap.create({});

console.log(asdf);
console.log(InfrastructureMap.toJson(asdf));
