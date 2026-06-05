/**
 * On-device app icon renderer.
 * Run: CLASSPATH=/data/local/tmp/adbph_icons.dex app_process / AppIcons [comma_separated_pkgs]
 *
 * Uses 100% reflection to avoid compile-time Android SDK dependency.
 * If package names are provided as a single comma-separated argument, only renders those icons
 * (incremental mode). Otherwise, renders all installed app icons (batch mode).
 *
 * Output format (one line per app): pkg|versionCode|base64
 *   - pkg: package name
 *   - versionCode: integer version code (defaults to 0 if unavailable)
 *   - base64: PNG bytes encoded with Base64.NO_WRAP (no line breaks)
 *
 * Lines starting with "ERROR:" or empty lines should be skipped by the host parser.
 */
public class AppIcons {
    public static void main(String[] args) {
        try {
            // Looper.prepareMainLooper()
            Class.forName("android.os.Looper")
                .getMethod("prepareMainLooper")
                .invoke(null);

            // ActivityThread.systemMain()
            Class<?> atClass = Class.forName("android.app.ActivityThread");
            Object thread = atClass.getMethod("systemMain").invoke(null);
            Object ctx = atClass.getMethod("getSystemContext").invoke(thread);

            // ctx.getPackageManager()
            Object pm = ctx.getClass().getMethod("getPackageManager").invoke(ctx);

            // Create output file on device
            String outputPath = "/data/local/tmp/adbph_icons.txt";
            Class<?> fwClass = Class.forName("java.io.FileWriter");
            Object writer = fwClass.getConstructor(String.class).newInstance(outputPath);
            Object bw = Class.forName("java.io.BufferedWriter")
                .getConstructor(Class.forName("java.io.Writer"))
                .newInstance(writer);

            // Determine target packages
            java.util.Set<String> targetPkgs = null;
            if (args.length > 0) {
                targetPkgs = new java.util.HashSet<>(
                    java.util.Arrays.asList(args[0].split(","))
                );
            }

            Class<?> appInfoClass = Class.forName("android.content.pm.ApplicationInfo");

            if (targetPkgs != null) {
                // Incremental mode — only render requested packages
                Class<?> pmClass = pm.getClass();
                java.lang.reflect.Method getAppInfoMethod = pmClass.getMethod(
                    "getApplicationInfo", String.class, int.class
                );
                for (String pkg : targetPkgs) {
                    try {
                        Object appInfo = getAppInfoMethod.invoke(pm, pkg, 0);
                        renderIcon(pm, appInfoClass, pmClass, appInfo, pkg, bw);
                    } catch (Exception e) {
                        // Package uninstalled since list — skip
                        System.err.println("SKIP:" + pkg + ":" + e.getMessage());
                    }
                }
            } else {
                // Batch mode — all installed apps
                int flags = 0x00000080; // GET_META_DATA
                java.util.List apps = (java.util.List) pm.getClass()
                    .getMethod("getInstalledApplications", int.class)
                    .invoke(pm, flags);
                for (Object app : apps) {
                    try {
                        String pkg = (String) app.getClass().getField("packageName").get(app);
                        renderIcon(pm, appInfoClass, pm.getClass(), app, pkg, bw);
                    } catch (Exception e) {
                        System.err.println("SKIP:" + e.getMessage());
                    }
                }
            }

            // Flush and close
            bw.getClass().getMethod("flush").invoke(bw);
            bw.getClass().getMethod("close").invoke(bw);

        } catch (Exception e) {
            System.err.println("ERROR:" + e.getClass().getName() + ":" + e.getMessage());
            System.exit(1);
        }
    }

    static void renderIcon(Object pm, Class<?> appInfoClass, Class<?> pmClass,
                           Object appInfo, String pkg, Object bw) throws Exception {
        // Get versionCode via PackageInfo (reflection)
        int versionCode = 0;
        try {
            Object pkgInfo = pmClass.getMethod("getPackageInfo", String.class, int.class)
                .invoke(pm, pkg, 0);
            versionCode = (Integer) pkgInfo.getClass().getField("versionCode").get(pkgInfo);
        } catch (Exception e) {
            // versionCode unavailable — default to 0
        }

        // Get application icon
        Object drawable = pmClass.getMethod("getApplicationIcon", appInfoClass)
            .invoke(pm, appInfo);

        // Convert drawable to bitmap
        Object bitmap = drawableToBitmap(drawable);

        // Compress to PNG
        byte[] png = compressToPng(bitmap);

        // Encode to base64 (NO_WRAP = 2)
        String b64 = (String) Class.forName("android.util.Base64")
            .getMethod("encodeToString", byte[].class, int.class)
            .invoke(null, png, 2);

        // Write: pkg|versionCode|base64
        bw.getClass().getMethod("write", String.class)
            .invoke(bw, pkg + "|" + versionCode + "|" + b64);
        bw.getClass().getMethod("newLine").invoke(bw);

        // Recycle bitmap
        bitmap.getClass().getMethod("recycle").invoke(bitmap);
    }

    static Object drawableToBitmap(Object drawable) throws Exception {
        int w = 96, h = 96;

        Class<?> bitmapClass = Class.forName("android.graphics.Bitmap");
        Class<?> configClass = Class.forName("android.graphics.Bitmap$Config");
        Object argb8888 = configClass.getField("ARGB_8888").get(null);

        // Create bitmap + canvas with circular clip for consistent look
        Object bm = bitmapClass.getMethod("createBitmap", int.class, int.class, configClass)
            .invoke(null, w, h, argb8888);

        Object canvas = Class.forName("android.graphics.Canvas")
            .getConstructor(bitmapClass)
            .newInstance(bm);

        // Apply circular clip — ensures consistent icon shape regardless of device
        Object path = Class.forName("android.graphics.Path").newInstance();
        Class<?> directionClass = Class.forName("android.graphics.Path$Direction");
        path.getClass().getMethod("addCircle", float.class, float.class, float.class,
            directionClass)
            .invoke(path, w / 2f, h / 2f, h / 2f,
                directionClass.getField("CW").get(null));
        canvas.getClass().getMethod("clipPath", path.getClass())
            .invoke(canvas, path);

        // Set drawable bounds and draw — works for ALL drawable types:
        // AdaptiveIconDrawable: draws bg + fg layers with proper scaling
        // BitmapDrawable: draws the bitmap
        // Other Drawable: draws via generic draw()
        drawable.getClass().getMethod("setBounds", int.class, int.class, int.class, int.class)
            .invoke(drawable, 0, 0, w, h);
        drawable.getClass().getMethod("draw", canvas.getClass()).invoke(drawable, canvas);

        return bm;
    }

    static byte[] compressToPng(Object bitmap) throws Exception {
        Object baos = Class.forName("java.io.ByteArrayOutputStream").newInstance();
        Class<?> compressFormatClass = Class.forName("android.graphics.Bitmap$CompressFormat");
        Object pngFormat = compressFormatClass.getField("PNG").get(null);
        bitmap.getClass().getMethod("compress",
            compressFormatClass,
            int.class,
            Class.forName("java.io.OutputStream"))
            .invoke(bitmap, pngFormat, 100, baos);
        return (byte[]) baos.getClass().getMethod("toByteArray").invoke(baos);
    }
}