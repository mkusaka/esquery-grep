interface Config {
  host: string;
  port: number;
}

function createServer(config: Config) {
  return { listen: () => console.log(config.host) };
}

const val = 1 + 2 + 3;
