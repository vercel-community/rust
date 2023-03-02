import random from 'random';
// @ts-expect-error
import shishua from 'shishua';
import type { NextApiRequest, NextApiResponse } from 'next';

interface Data {
  runtime: 'node';
  message: string;
  time: string;
  pi: number;
}

export default function handler(
  _req: NextApiRequest,
  res: NextApiResponse<Data>,
): void {
  const t0 = performance.now();
  random.use(shishua('seed'));

  const radius = 424242;
  const loops = 10_000_000;
  let counter = 0;

  for (let i = 0; i < loops; i++) {
    const x = random.int(1, radius);
    const y = random.int(1, radius);

    if (Math.pow(x, 2) + Math.pow(y, 2) < Math.pow(radius, 2)) {
      counter++;
    }
  }

  const pi = (4.0 * counter) / loops;

  const t1 = performance.now();

  res.status(200).json({
    runtime: 'node',
    message: `${counter} of ${loops} points within circle district`,
    time: `${(t1 - t0).toFixed(2)} milliseconds`,
    pi,
  });
}
