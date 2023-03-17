import { generateRoutes } from './routes';

describe('generateRoutes', () => {
  it('should generate static routes', () => {
    const staticRoutes = ['api/foo.rs', 'api/bar/baz.rs'];

    expect(generateRoutes(staticRoutes)).toMatchInlineSnapshot(`
      [
        {
          "dest": "/api/vercel/index",
          "src": "/api/bar/baz",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/foo",
        },
      ]
    `);
  });

  it('should generate dynamic routes', () => {
    const dynamicRoutes = [
      'api/post/[id].rs',
      'api/post/[id]/comments/[commentId].rs',
    ];

    expect(generateRoutes(dynamicRoutes)).toMatchInlineSnapshot(`
      [
        {
          "dest": "/api/vercel/index?id=$id&commentId=$commentId",
          "src": "/api/post/(?<id>[^/]+)/comments/(?<commentId>[^/]+)",
        },
        {
          "dest": "/api/vercel/index?id=$id",
          "src": "/api/post/(?<id>[^/]+)",
        },
      ]
    `);
  });

  it('should generate catch-all routes', () => {
    const catchAllRoutes = [
      'api/[...rootAll].rs',
      'api/all/[...all].rs',
      'api/optional/[[...id]].rs',
    ];

    expect(generateRoutes(catchAllRoutes)).toMatchInlineSnapshot(`
      [
        {
          "dest": "/api/vercel/index",
          "src": "/api/all/(\\S+)",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/optional/(/\\S+)?",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/(\\S+)",
        },
      ]
    `);
  });

  it('should sort all routes correctly', () => {
    const allRoutes = [
      'api/foo.rs',
      'api/bar/baz.rs',
      'api/post/[id].rs',
      'api/post/[id]/comments/[commentId].rs',
      'api/[...rootAll].rs',
      'api/all/[...all].rs',
      'api/optional/[[...id]].rs',
    ];

    expect(generateRoutes(allRoutes)).toMatchInlineSnapshot(`
      [
        {
          "dest": "/api/vercel/index",
          "src": "/api/bar/baz",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/foo",
        },
        {
          "dest": "/api/vercel/index?id=$id&commentId=$commentId",
          "src": "/api/post/(?<id>[^/]+)/comments/(?<commentId>[^/]+)",
        },
        {
          "dest": "/api/vercel/index?id=$id",
          "src": "/api/post/(?<id>[^/]+)",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/all/(\\S+)",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/optional/(/\\S+)?",
        },
        {
          "dest": "/api/vercel/index",
          "src": "/api/(\\S+)",
        },
      ]
    `);
  });
});
