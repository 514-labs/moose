import {
  InfrastructureMap,
  InfrastructureMapSchema,
} from "./gen/infrastructure_map_pb.js";
import { create, toBinary, toJson, fromBinary } from "@bufbuild/protobuf";

const asdf: InfrastructureMap = fromBinary(
  InfrastructureMapSchema,
  Uint8Array.of(),
);
console.log(asdf);
console.log(toJson(InfrastructureMapSchema, asdf));
