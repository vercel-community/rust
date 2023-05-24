import { orderBy } from 'lodash';

const CatchPriority = {
  Static: 0,
  Dynamic: 1,
  CatchAll: 2,
  OptionalCatchAll: 2,
};

interface Route {
  src: string;
  dest: string;
}

export function generateRoutes(files: string[]): Route[] {
  const routes = files.map((key) => {
    const route = key.endsWith('.rs') ? key.slice(0, -3) : key;
    const segments = route.split('/');

    const result = segments.reduce<{
      catchType: null | number;
      src: string[];
      searchParams: URLSearchParams;
    }>(
      (acc, segment) => {
        // Catch all route
        if (segment.startsWith('[...') && segment.endsWith(']')) {
          acc.catchType = CatchPriority.CatchAll;
          acc.src.push('(\\S+)');
          return acc;
        }

        // Optional catch all route
        if (segment.startsWith('[[...') && segment.endsWith(']]')) {
          acc.catchType = CatchPriority.OptionalCatchAll;
          acc.src.push('(/\\S+)?');
          return acc;
        }

        // Dynamic route
        if (segment.startsWith('[') && segment.endsWith(']')) {
          const parameterName = segment.replace('[', '').replace(']', '');
          acc.catchType = CatchPriority.Dynamic;
          acc.src.push(`(?<${parameterName}>[^/]+)`);
          acc.searchParams.set(parameterName, `$${parameterName}`);
          return acc;
        }

        // Static route
        acc.catchType = CatchPriority.Static;
        acc.src.push(segment);

        return acc;
      },
      {
        catchType: null,
        src: [],
        searchParams: new URLSearchParams(),
      },
    );

    const searchParams = decodeURIComponent(result.searchParams.toString());
    const queryString = searchParams !== '' ? `?${searchParams}` : '';

    return {
      src: `/${result.src.join('/')}`,
      dest: `/api/main${queryString}`,
      depth: segments.length,
      catchType: result.catchType,
    };
  });

  const orderedRoutes = orderBy(
    routes,
    ['catchType', 'depth'],
    ['asc', 'desc'],
  );

  return orderedRoutes.map<Route>((r) => ({
    src: r.src,
    dest: r.dest,
  }));
}
