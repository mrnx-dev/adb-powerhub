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
                    Object inputStream = null;
                    try {
                        inputStream = zfClass.getMethod("getInputStream",
                            Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);

                        byte[] iconBytes = readAllBytes(inputStream);

                        if (iconBytes != null && iconBytes.length > 0) {
                            return (String) Class.forName("android.util.Base64")
                                .getMethod("encodeToString", byte[].class, int.class)
                                .invoke(null, iconBytes, 2);
                        }
                    } finally {
                        if (inputStream != null) {
                            inputStream.getClass().getMethod("close").invoke(inputStream);
                        }
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
     * Quick-scan AndroidManifest.xml (AXML binary) for icon drawable names.
     * Scans raw bytes for ASCII and UTF-16 strings that look like drawable names.
     * No full AXML parser needed.
     */
    static java.util.List<String> findIconNamesFromManifest(Object zf) throws Exception {
        Class<?> zfClass = zf.getClass();
        java.util.List<String> names = new java.util.ArrayList<>();

        Object entry = zfClass.getMethod("getEntry", String.class)
            .invoke(zf, "AndroidManifest.xml");
        if (entry == null) return names;

        Object is = zfClass.getMethod("getInputStream",
            Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);
        byte[] bytes = readAllBytes(is);
        is.getClass().getMethod("close").invoke(is);

        if (bytes == null || bytes.length < 100) return names;

        // Scan for ASCII strings
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < bytes.length; i++) {
            byte b = bytes[i];
            if (b >= 'a' && b <= 'z' || b >= 'A' && b <= 'Z'
                || b >= '0' && b <= '9' || b == '_' || b == '.' || b == '/') {
                sb.append((char) b);
            } else {
                if (sb.length() >= 3 && sb.length() <= 60) {
                    addIfIconName(names, sb.toString());
                }
                sb = new StringBuilder();
            }
        }
        if (sb.length() >= 3) addIfIconName(names, sb.toString());

        // Scan for UTF-16 strings (common in AXML string pools)
        for (int i = 0; i < bytes.length - 3; i += 2) {
            char c = (char) ((bytes[i] & 0xFF) | ((bytes[i+1] & 0xFF) << 8));
            if (c >= 'a' && c <= 'z' || c >= 'A' && c <= 'Z') {
                sb = new StringBuilder();
                sb.append(c);
                for (int j = i + 2; j < bytes.length - 1; j += 2) {
                    char c2 = (char) ((bytes[j] & 0xFF) | ((bytes[j+1] & 0xFF) << 8));
                    if (c2 >= 'a' && c2 <= 'z' || c2 >= 'A' && c2 <= 'Z'
                        || c2 >= '0' && c2 <= '9' || c2 == '_' || c2 == '.' || c2 == '/') {
                        sb.append(c2);
                    } else {
                        break;
                    }
                }
                if (sb.length() >= 3 && sb.length() <= 60) {
                    addIfIconName(names, sb.toString());
                }
                i += sb.length() * 2;
            }
        }

        return names;
    }

    static void addIfIconName(java.util.List<String> names, String s) {
        String lower = s.toLowerCase();
        // Must be a plausible drawable reference
        boolean isMipmap = lower.contains("mipmap") || lower.contains("drawable");
        boolean isIcon = lower.contains("ic_") || lower.contains("icon") || lower.contains("logo");
        boolean hasUnderscore = lower.contains("_");
        if (!isMipmap && !isIcon && !hasUnderscore) return;

        // Extract just the base name (strip path and extension)
        String name = s;
        int slash = name.lastIndexOf('/');
        if (slash >= 0) name = name.substring(slash + 1);
        int dot = name.lastIndexOf('.');
        if (dot > 0 && dot < name.length() - 1) {
            name = name.substring(0, dot);
        }
        if (!names.contains(name)) {
            names.add(name);
        }
    }

    /**
     * Search the APK ZIP for the best launcher icon.
     * First checks AndroidManifest.xml for icon drawable names (AXML quick-scan),
     * then scans all PNG entries in mipmap directories, prefers higher density
     * and filenames matching manifest hints or containing ic_launcher / icon.
     */
    static String findBestIcon(Object zf) throws Exception {
        Class<?> zfClass = zf.getClass();

        // Collect all PNG entries in icon directories
        java.util.List<String> candidates = new java.util.ArrayList<>();

        // Get all entries via reflection: Enumeration<? extends ZipEntry> entries()
        Object entries = zfClass.getMethod("entries").invoke(zf);
        Class<?> enumClass = Class.forName("java.util.Enumeration");
        while ((Boolean) enumClass.getMethod("hasMoreElements").invoke(entries)) {
            Object entry = enumClass.getMethod("nextElement").invoke(entries);
            String name = (String) entry.getClass().getMethod("getName").invoke(entry);
            if (name == null) continue;

            // Only look at PNG files in mipmap directories (launcher icons)
            if (!name.endsWith(".png")) continue;
            if (!name.toLowerCase().contains("mipmap")) continue;

            candidates.add(name);
        }

        if (candidates.isEmpty()) return null;

        // Quick-scan AndroidManifest for icon name hints
        java.util.List<String> manifestHints = findIconNamesFromManifest(zf);

        // Score each candidate: prefer ic_launcher > icon, prefer higher density,
        // bonus for matching manifest hints
        String best = null;
        int bestScore = -1;
        for (String c : candidates) {
            int score = scoreIcon(c, manifestHints);
            if (score > bestScore) {
                bestScore = score;
                best = c;
            }
        }

        return best;
    }

    static int scoreIcon(String path, java.util.List<String> manifestHints) {
        String lower = path.toLowerCase();
        int score = 0;

        // Density bonus (higher density = higher score)
        if (lower.contains("xxxhdpi")) score += 600;
        else if (lower.contains("xxhdpi")) score += 500;
        else if (lower.contains("xhdpi")) score += 400;
        else if (lower.contains("hdpi")) score += 300;
        else if (lower.contains("mdpi")) score += 200;
        else if (lower.contains("ldpi")) score += 100;

        // Name bonus: prefer launcher icons
        if (lower.contains("ic_launcher")) {
            score += 1000;
            if (!lower.contains("round") && !lower.contains("foreground") && !lower.contains("background")) {
                score += 500;
            }
        } else if (lower.contains("icon")) {
            score += 500;
        }

        // Bonus: filename matches manifest hint — very strong signal!
        if (manifestHints != null) {
            for (String hint : manifestHints) {
                if (lower.contains(hint.toLowerCase())) {
                    score += 2000; // Manifest says this is the icon!
                    break;
                }
            }
        }

        // Prefer mipmap over drawable
        if (lower.contains("mipmap")) score += 200;

        return score;
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
