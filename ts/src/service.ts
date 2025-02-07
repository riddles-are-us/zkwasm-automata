import { Service } from "zkwasm-ts-server";

const service = new Service(
  async (arg, events) => { return; },
  async () => { return; }
);
service.initialize();
service.serve();


