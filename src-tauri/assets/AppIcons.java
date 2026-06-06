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
    private static boolean looperPrepared = false;

    static synchronized void ensureLooper() throws Exception {
        if (!looperPrepared) {
            Class.forName("android.os.Looper")
                .getMethod("prepareMainLooper")
                .invoke(null);
            looperPrepared = true;
        }
    }

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
     *
     * Strategy (tried in order):
     *   1. PackageManager.getPackageArchiveInfo → resource ID
     *   2. Parse resources.arsc → resolve ID to drawable name
     *   3. Search ZIP for matching PNG
     *   4. Fallback: ZIP scan + AXML hints + scoring
     *   5. Last resort: PackageManager.getApplicationIcon via app_process
     */
    static String extractIconFromApk(String apkPath) throws Exception {
        Class<?> zfClass = Class.forName("java.util.zip.ZipFile");
        Object zf = zfClass.getConstructor(String.class).newInstance(apkPath);

        try {
            // ── Method 1: Resource ID via PackageManager + resources.arsc ──
            String iconName = resolveIconNameFromArsc(zf, apkPath);
            if (iconName != null && !iconName.isEmpty()) {
                String result = findAndReadIcon(zf, iconName);
                if (result != null) return result;
            }

            // ── Method 2: ZIP scan + AXML hints + scoring ──
            String bestIcon = findBestIcon(zf);
            if (bestIcon != null) {
                Object entry = zfClass.getMethod("getEntry", String.class).invoke(zf, bestIcon);
                if (entry != null) {
                    Object is = zfClass.getMethod("getInputStream",
                        Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);
                    byte[] bytes = readAllBytes(is);
                    is.getClass().getMethod("close").invoke(is);
                    if (bytes != null && bytes.length > 0) {
                        return (String) Class.forName("android.util.Base64")
                            .getMethod("encodeToString", byte[].class, int.class)
                            .invoke(null, bytes, 2);
                    }
                }
            }
        } catch (Exception e) {
            System.err.println("ZIP:" + apkPath + ":" + e.getMessage());
        } finally {
            zfClass.getMethod("close").invoke(zf);
        }

        // ── Method 3: PackageManager fallback ──
        return extractViaPackageManager(apkPath);
    }

    /**
     * Resolve the icon drawable name using PackageManager + resources.arsc.
     * This is the most accurate method: gets the resource ID from PM,
     * resolves it to a name via ARSC parsing, then searches the ZIP.
     */
    static String resolveIconNameFromArsc(Object zf, String apkPath) {
        try {
            // Step 1: Get resource ID via PackageManager.getPackageArchiveInfo
            int iconResId = getIconResourceId(apkPath);
            if (iconResId == 0) return null;

            // Step 2: Read resources.arsc from the APK
            Class<?> zfClass = zf.getClass();
            Object arscEntry = zfClass.getMethod("getEntry", String.class)
                .invoke(zf, "resources.arsc");
            if (arscEntry == null) return null;

            Object is = zfClass.getMethod("getInputStream",
                Class.forName("java.util.zip.ZipEntry")).invoke(zf, arscEntry);
            byte[] arsc = readAllBytes(is);
            is.getClass().getMethod("close").invoke(is);

            if (arsc == null || arsc.length < 20) return null;

            // Step 3: Resolve resource ID → type name + entry name
            return resolveArsc(arsc, iconResId);
        } catch (Exception e) {
            System.err.println("ARSC:" + e.getMessage());
            return null;
        }
    }

    /**
     * Get the icon resource ID from the APK using PackageManager.
     * Uses getPackageArchiveInfo which reads the manifest directly — no permission issues.
     */
    static int getIconResourceId(String apkPath) throws Exception {
        ensureLooper();
        Class<?> atClass = Class.forName("android.app.ActivityThread");
        Object thread = atClass.getMethod("systemMain").invoke(null);
        Object ctx = atClass.getMethod("getSystemContext").invoke(thread);
        Object pm = ctx.getClass().getMethod("getPackageManager").invoke(ctx);

        Object pkgInfo = pm.getClass()
            .getMethod("getPackageArchiveInfo", String.class, int.class)
            .invoke(pm, apkPath, 0);
        if (pkgInfo == null) return 0;

        Object appInfo = pkgInfo.getClass().getField("applicationInfo").get(pkgInfo);
        // Set source dir so resources can be found
        appInfo.getClass().getField("sourceDir").set(appInfo, apkPath);
        appInfo.getClass().getField("publicSourceDir").set(appInfo, apkPath);

        return ((Integer) appInfo.getClass().getField("icon").get(appInfo)).intValue();
    }

    /**
     * Parse resources.arsc binary to resolve a resource ID to a drawable name.
     *
     * Resource ID breakdown: 0xPPTTEEEE
     *   PP = package ID (0x7F for app's own resources)
     *   TT = type ID (index into type strings pool)
     *   EEEE = entry index (index into key strings pool)
     *
     * resources.arsc structure (simplified):
     *   [Header 12B] [Global StringPool] [Package...]
     *   Package: [Header] [TypeStrings] [KeyStrings] [TypeSpecs...]
     */
    static String resolveArsc(byte[] data, int resId) {
        int packageId = (resId >> 24) & 0xFF;
        int typeId = (resId >> 16) & 0xFF;
        int entryIndex = resId & 0xFFFF;

        int pos = 0;

        // Read header
        if (data.length < 12) return null;
        int packageCount = readInt32(data, 8);
        pos = readInt32(data, 4); // header size

        // Skip global string pool
        pos = skipStringPool(data, pos);
        if (pos <= 0 || pos >= data.length) return null;

        // Find matching package
        for (int p = 0; p < packageCount; p++) {
            if (pos + 12 > data.length) return null;
            int pkgType = readInt32(data, pos);
            int pkgHeaderSize = readInt32(data, pos + 4);
            int pkgId = readInt32(data, pos + 8);

            if (pkgId == packageId) {
                return resolveInPackage(data, pos, pkgHeaderSize, typeId, entryIndex);
            }
            pos += pkgHeaderSize;
        }

        return null; // package not found
    }

    static int skipStringPool(byte[] data, int pos) {
        if (pos + 4 > data.length) return -1;
        int type = readInt32(data, pos);
        if (type != 0x001C0001) return pos; // not a string pool
        return pos + readInt32(data, pos + 8); // skip total_size bytes
    }

    static String resolveInPackage(byte[] data, int pkgStart, int pkgHeaderSize,
                                   int typeId, int entryIndex) {
        // Package header fields at fixed offsets
        int typeStringsOff = readInt32(data, pkgStart + 268); // offset from pkgStart
        int keyStringsOff  = readInt32(data, pkgStart + 276); // offset from pkgStart

        // Read type strings pool → get type name for typeId
        int typePoolPos = pkgStart + typeStringsOff;
        String typeName = readStringFromPool(data, typePoolPos, typeId);
        if (typeName == null || typeName.isEmpty()) return null;

        // Read key strings pool → get entry name for entryIndex
        int keyPoolPos = pkgStart + keyStringsOff;
        String entryName = readStringFromPool(data, keyPoolPos, entryIndex);
        if (entryName == null || entryName.isEmpty()) return null;

        return entryName;
    }

    /**
     * Read a string from a ResStringPool at a given index.
     */
    static String readStringFromPool(byte[] data, int poolStart, int index) {
        if (poolStart + 28 > data.length) return null;

        int stringCount = readInt32(data, poolStart + 12);
        if (index < 0 || index >= stringCount) return null;

        int flags = readInt32(data, poolStart + 20);
        int stringsStart = readInt32(data, poolStart + 24);
        int headerSize = readInt32(data, poolStart + 4);

        // String offset table starts at poolStart + headerSize
        int offsetTablePos = poolStart + headerSize;
        int strOffset = readInt32(data, offsetTablePos + index * 4);

        // String data starts at poolStart + stringsStart
        int strPos = poolStart + stringsStart + strOffset;

        boolean isUtf8 = (flags & 0x100) != 0;

        if (isUtf8) {
            return readUtf8String(data, strPos);
        } else {
            return readUtf16String(data, strPos);
        }
    }

    static String readUtf8String(byte[] data, int pos) {
        if (pos >= data.length) return null;
        int len = data[pos] & 0xFF;
        int charLen;
        if ((len & 0x80) != 0) {
            // 2-byte length
            if (pos + 1 >= data.length) return null;
            len = ((len & 0x7F) << 8) | (data[pos + 1] & 0xFF);
            charLen = 2;
        } else {
            charLen = 1;
        }
        int dataStart = pos + charLen;
        int dataEnd = dataStart + len;
        if (dataEnd > data.length) return null;

        // Find actual end (null terminator or end of readable data)
        int end = dataStart;
        while (end < dataEnd && data[end] != 0) end++;

        try {
            return new String(data, dataStart, end - dataStart, "UTF-8");
        } catch (Exception e) {
            return null;
        }
    }

    static String readUtf16String(byte[] data, int pos) {
        if (pos + 1 >= data.length) return null;
        int len = ((data[pos] & 0xFF) | ((data[pos + 1] & 0xFF) << 8));
        int charLen;
        if ((len & 0x8000) != 0) {
            // 4-byte length (2 uint16)
            if (pos + 3 >= data.length) return null;
            len = ((len & 0x7FFF) << 16)
                | ((data[pos + 2] & 0xFF) | ((data[pos + 3] & 0xFF) << 8));
            charLen = 4;
        } else {
            charLen = 2;
        }

        int dataStart = pos + charLen;
        int dataEnd = dataStart + len * 2; // len is in uint16 units
        if (dataEnd > data.length) return null;

        // Decode UTF-16LE manually
        StringBuilder sb = new StringBuilder();
        for (int i = dataStart; i < dataEnd - 1; i += 2) {
            char c = (char) ((data[i] & 0xFF) | ((data[i + 1] & 0xFF) << 8));
            if (c == 0) break; // null terminator
            sb.append(c);
        }
        return sb.toString();
    }

    /**
     * Read a little-endian 32-bit integer from a byte array.
     */
    static int readInt32(byte[] data, int offset) {
        return (data[offset] & 0xFF)
            | ((data[offset + 1] & 0xFF) << 8)
            | ((data[offset + 2] & 0xFF) << 16)
            | ((data[offset + 3] & 0xFF) << 24);
    }

    /**
     * Search ZIP for a PNG matching the resolved drawable name.
     * Tries the name in all mipmap density directories.
     */
    static String findAndReadIcon(Object zf, String drawableName) throws Exception {
        Class<?> zfClass = zf.getClass();
        String[] densities = {"xxxhdpi", "xxhdpi", "xhdpi", "hdpi", "mdpi"};

        for (String density : densities) {
            // Try with -v4 suffix
            String path = "res/mipmap-" + density + "-v4/" + drawableName + ".png";
            Object entry = zfClass.getMethod("getEntry", String.class).invoke(zf, path);
            if (entry != null) {
                Object is = zfClass.getMethod("getInputStream",
                    Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);
                byte[] bytes = readAllBytes(is);
                is.getClass().getMethod("close").invoke(is);
                if (bytes != null && bytes.length > 0) {
                    return (String) Class.forName("android.util.Base64")
                        .getMethod("encodeToString", byte[].class, int.class)
                        .invoke(null, bytes, 2);
                }
            }
            // Try without -v4 suffix
            path = "res/mipmap-" + density + "/" + drawableName + ".png";
            entry = zfClass.getMethod("getEntry", String.class).invoke(zf, path);
            if (entry != null) {
                Object is = zfClass.getMethod("getInputStream",
                    Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);
                byte[] bytes = readAllBytes(is);
                is.getClass().getMethod("close").invoke(is);
                if (bytes != null && bytes.length > 0) {
                    return (String) Class.forName("android.util.Base64")
                        .getMethod("encodeToString", byte[].class, int.class)
                        .invoke(null, bytes, 2);
                }
            }
        }

        // Also try anydpi-v26 (contains adaptive icon XML)
        String path = "res/mipmap-anydpi-v26/" + drawableName + ".xml";
        Object entry = zfClass.getMethod("getEntry", String.class).invoke(zf, path);
        if (entry != null) {
            // Adaptive icon → extract foreground from nearest density
            // For now, fall back to foreground PNG
            for (String density : densities) {
                String fgPath = "res/mipmap-" + density + "-v4/" + drawableName + "_foreground.png";
                entry = zfClass.getMethod("getEntry", String.class).invoke(zf, fgPath);
                if (entry != null) {
                    Object is = zfClass.getMethod("getInputStream",
                        Class.forName("java.util.zip.ZipEntry")).invoke(zf, entry);
                    byte[] bytes = readAllBytes(is);
                    is.getClass().getMethod("close").invoke(is);
                    if (bytes != null && bytes.length > 0) {
                        return (String) Class.forName("android.util.Base64")
                            .getMethod("encodeToString", byte[].class, int.class)
                            .invoke(null, bytes, 2);
                    }
                }
            }
        }

        return null;
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
            ensureLooper();
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
