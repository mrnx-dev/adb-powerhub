/**
 * On-device app label resolver.
 * Run: CLASSPATH=/data/local/tmp/adbph_labels.dex app_process / AppLabels
 *
 * Uses 100% reflection to avoid compile-time Android SDK dependency.
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

            // GET_META_DATA = 0x00000080
            int flags = 0x00000080;
            // pm.getInstalledApplications(flags)
            java.util.List apps = (java.util.List) pm.getClass()
                .getMethod("getInstalledApplications", int.class)
                .invoke(pm, flags);

            for (Object app : apps) {
                String pkg = (String) app.getClass().getField("packageName").get(app);
                // pm.getApplicationLabel(app)
                CharSequence label = (CharSequence) pm.getClass()
                    .getMethod("getApplicationLabel", Class.forName("android.content.pm.ApplicationInfo"))
                    .invoke(pm, app);
                System.out.println(pkg + "|" + label);
            }
        } catch (Exception e) {
            System.err.println("ERROR:" + e.getClass().getName() + ":" + e.getMessage());
            System.exit(1);
        }
    }
}
