import * as fs from 'node:fs';
import * as path from 'node:path';

export const metafilePlugin = {
  name: 'metafile-plugin',
  setup: (build) => {
    build.onEnd(async (result) => {
      if (result.metafile == null) {
        return;
      }

      await fs.promises.writeFile(
        path.join(__dirname, '../../metafile.json'),
        JSON.stringify(result.metafile, null, 2),
        'utf-8',
      );
    });
  },
};
