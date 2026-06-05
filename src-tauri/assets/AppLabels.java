/**
 * On-device app label resolver.
 * Run: CLASSPATH=/data/local/tmp/adbph_labels.dex app_process / AppLabels [package_name]
 *
 * Uses 100% reflection to avoid compile-time Android SDK dependency.
 * If package_name is provided, resolves only that package via getApplicationInfo.
 * Otherwise, resolves all installed apps via getInstalledApplications.
 */
public class AppLabels {
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

            Class<?> appInfoClass = Class.forName("android.content.pm.ApplicationInfo");

            if (args.length > 0) {
                // Single-package lookup: faster, no loop
                String targetPkg = args[0];
                Object appInfo = pm.getClass()
                    .getMethod("getApplicationInfo", String.class, int.class)
                    .invoke(pm, targetPkg, 0);
                CharSequence label = (CharSequence) pm.getClass()
                    .getMethod("getApplicationLabel", appInfoClass)
                    .invoke(pm, appInfo);
                System.out.println(targetPkg + "|" + label);
            } else {
                // Batch lookup: all installed apps
                int flags = 0x00000080; // GET_META_DATA
                java.util.List apps = (java.util.List) pm.getClass()
                    .getMethod("getInstalledApplications", int.class)
                    .invoke(pm, flags);

                for (Object app : apps) {
                    String pkg = (String) app.getClass().getField("packageName").get(app);
                    CharSequence label = (CharSequence) pm.getClass()
                        .getMethod("getApplicationLabel", appInfoClass)
                        .invoke(pm, app);
                    System.out.println(pkg + "|" + label);
                }
            }
        } catch (Exception e) {
            System.err.println("ERROR:" + e.getClass().getName() + ":" + e.getMessage());
            System.exit(1);
        }
    }
}
