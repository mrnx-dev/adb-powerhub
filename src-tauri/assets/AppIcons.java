/**
 * On-device app icon extractor — reads icons directly from APK ZIP files.
 * Run: CLASSPATH=/data/local/tmp/adbph_icons.dex app_process / AppIcons pkg1=apk1,pkg2=apk2,...
 *
 * Unlike the old PackageManager.getApplicationIcon() approach (which returned
 * default icons for third-party apps due to shell UID restrictions), this
 * version opens each APK as a ZipFile and reads the actual icon bytes directly
 * from the filesystem. Shell user CAN read APK files in /data/app/ and /system/.
 *
 * Uses 100% reflection to avoid compile-time Android SDK dependency.
 *
 * Input: comma-separated "pkg=apk_path" pairs
 * Output (stdout): pkg|base64 (one per line)
 * Lines starting with "ERROR:" should be skipped by the host parser.
 */
public class AppIcons {
    public static void main(String[] args) {
        if (args.length == 0) {
            System.err.println("ERROR:Usage: AppIcons pkg1=apk1,pkg2=apk2,...");
            System.exit(1);
        }

        try {
            // Parse input: "com.a=/data/app/.../base.apk,com.b=/system/app/.../app.apk"
            String[] entries = args[0].split(",");
            for (String entry : entries) {
                int eq = entry.indexOf('=');
                if (eq <= 0) continue;
                String pkg = entry.substring(0, eq);
                String apkPath = entry.substring(eq + 1);
                if (pkg.isEmpty() || apkPath.isEmpty()) continue;

                try {
                    String b64 = extractIconFromApk(apkPath);
                    if (b64 != null && !b64.isEmpty()) {
                        System.out.println(pkg + "|" + b64);
                    } else {
                        System.err.println("SKIP:" + pkg + ":no icon found");
                    }
                } catch (Exception e) {
                    System.err.println("SKIP:" + pkg + ":" + e.getMessage());
                }
            }
        } catch (Exception e) {
            System.err.println("ERROR:" + e.getClass().getName() + ":" + e.getMessage());
            System.exit(1);
        }
    }

    /**
     * Open an APK as a ZIP, find the best launcher icon, return as base64 PNG.
     * Falls back to PackageManager if ZIP reading fails.
     */
    static String extractIconFromApk(String apkPath) throws Exception {
        // Try ZIP-based extraction first (fast, reliable, no permission issues)
        try {
            // java.util.zip.ZipFile
            Class<?> zfClass = Class.forName("java.util.zip.ZipFile");
            Object zf = zfClass.getConstructor(String.class).newInstance(apkPath);

            // Find the best icon entry
            String bestIcon = findBestIcon(zf);
            if (bestIcon != null) {
                // Read entry bytes
                Object entry = zfClass.getMethod("getEntry", String.class).invoke(zf, bestIcon);
                if (entry != null) {
                    Object inputStream = zfClass.getMethod("getInputStream",
                        Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);

                    byte[] iconBytes = readAllBytes(inputStream);
                    inputStream.getClass().getMethod("close").invoke(inputStream);
                    zfClass.getMethod("close").invoke(zf);

                    if (iconBytes != null && iconBytes.length > 0) {
                        // Encode to base64 (NO_WRAP = 2)
                        return (String) Class.forName("android.util.Base64")
                            .getMethod("encodeToString", byte[].class, int.class)
                            .invoke(null, iconBytes, 2);
                    }
                }
            }
            zfClass.getMethod("close").invoke(zf);
        } catch (Exception e) {
            System.err.println("ZIP:" + apkPath + ":" + e.getMessage());
        }

        // Fallback: try PackageManager (works for system apps)
        return extractViaPackageManager(apkPath);
    }

    /**
     * Search the APK ZIP for the best launcher icon.
     * Priority: highest density first (xxxhdpi, xxhdpi, ...).
     */
    static String findBestIcon(Object zf) throws Exception {
        Class<?> zfClass = zf.getClass();

        // Density preference (highest first)
        String[] densities = {"xxxhdpi", "xxhdpi", "xhdpi", "hdpi", "mdpi"};
        String[] prefixes = {"mipmap", "drawable"};

        for (String density : densities) {
            for (String prefix : prefixes) {
                // Try with -v4 suffix: res/mipmap-xxxhdpi-v4/ic_launcher.png
                String dir = "res/" + prefix + "-" + density + "-v4/";
                String path = dir + "ic_launcher.png";
                Object entry = zfClass.getMethod("getEntry", String.class).invoke(zf, path);
                if (entry != null) return path;

                // Try without -v4 suffix
                dir = "res/" + prefix + "-" + density + "/";
                path = dir + "ic_launcher.png";
                entry = zfClass.getMethod("getEntry", String.class).invoke(zf, path);
                if (entry != null) return path;

                // Try ic_launcher_foreground (adaptive icon foreground)
                path = dir + "ic_launcher_foreground.png";
                entry = zfClass.getMethod("getEntry", String.class).invoke(zf, path);
                if (entry != null) return path;
            }
        }

        return null;
    }

    /**
     * Read all bytes from an InputStream using reflection.
     */
    static byte[] readAllBytes(Object inputStream) throws Exception {
        Class<?> isClass = inputStream.getClass();
        // Use ByteArrayOutputStream to collect bytes
        Object baos = Class.forName("java.io.ByteArrayOutputStream").newInstance();
        byte[] buf = new byte[8192];
        // read(byte[]) returns int
        while (true) {
            Object readCount = isClass.getMethod("read", byte[].class).invoke(inputStream, buf);
            int count = ((Integer) readCount).intValue();
            if (count < 0) break;
            baos.getClass().getMethod("write", byte[].class, int.class, int.class)
                .invoke(baos, buf, 0, count);
        }
        return (byte[]) baos.getClass().getMethod("toByteArray").invoke(baos);
    }

    /**
     * Fallback: try PackageManager.getApplicationIcon().
     * Only works for apps where shell user has resource access (system apps).
     */
    static String extractViaPackageManager(String apkPath) {
        try {
            // Initialize Android context
            Class.forName("android.os.Looper")
                .getMethod("prepareMainLooper")
                .invoke(null);
            Class<?> atClass = Class.forName("android.app.ActivityThread");
            Object thread = atClass.getMethod("systemMain").invoke(null);
            Object ctx = atClass.getMethod("getSystemContext").invoke(thread);
            Object pm = ctx.getClass().getMethod("getPackageManager").invoke(ctx);

            // Try to find package name from APK path
            // Use PackageManager.getPackageArchiveInfo() to read the APK's manifest
            Class<?> pmClass = pm.getClass();
            Object pkgInfo = pmClass.getMethod("getPackageArchiveInfo",
                String.class, int.class).invoke(pm, apkPath, 0);
            if (pkgInfo == null) return null;

            String pkgName = (String) pkgInfo.getClass().getField("packageName").get(pkgInfo);
            Object appInfo = pkgInfo.getClass().getField("applicationInfo").get(pkgInfo);

            // Set the source directory so the icon can be loaded
            appInfo.getClass().getField("sourceDir").set(appInfo, apkPath);
            appInfo.getClass().getField("publicSourceDir").set(appInfo, apkPath);

            Class<?> appInfoClass = Class.forName("android.content.pm.ApplicationInfo");
            Object drawable = pmClass.getMethod("getApplicationIcon", appInfoClass)
                .invoke(pm, appInfo);

            if (drawable == null) return null;

            // Render to bitmap
            Object bitmap = drawableToBitmap(drawable);
            byte[] png = compressToPng(bitmap);
            bitmap.getClass().getMethod("recycle").invoke(bitmap);

            return (String) Class.forName("android.util.Base64")
                .getMethod("encodeToString", byte[].class, int.class)
                .invoke(null, png, 2);
        } catch (Exception e) {
            return null;
        }
    }

    static Object drawableToBitmap(Object drawable) throws Exception {
        int w = 192, h = 192;
        Class<?> bitmapClass = Class.forName("android.graphics.Bitmap");
        Class<?> configClass = Class.forName("android.graphics.Bitmap$Config");
        Object argb8888 = configClass.getField("ARGB_8888").get(null);
        Object bm = bitmapClass.getMethod("createBitmap", int.class, int.class, configClass)
            .invoke(null, w, h, argb8888);
        Object canvas = Class.forName("android.graphics.Canvas")
            .getConstructor(bitmapClass).newInstance(bm);
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
            compressFormatClass, int.class, Class.forName("java.io.OutputStream"))
            .invoke(bitmap, pngFormat, 100, baos);
        return (byte[]) baos.getClass().getMethod("toByteArray").invoke(baos);
    }
}
