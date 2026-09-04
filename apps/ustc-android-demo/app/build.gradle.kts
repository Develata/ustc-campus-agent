plugins {
    id("com.android.application")
}

val sourceCommit = providers.gradleProperty("ucaSourceCommit").orElse("local").get()
require(sourceCommit == "local" || sourceCommit.matches(Regex("[0-9a-f]{40}"))) {
    "ucaSourceCommit must be 'local' or a lowercase 40-character Git SHA"
}

android {
    namespace = "com.develata.ustccampusagent"
    compileSdk = 36

    defaultConfig {
        applicationId = "com.develata.ustccampusagent"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0-mvp"
        buildConfigField("String", "SOURCE_COMMIT", "\"$sourceCommit\"")
        testInstrumentationRunner = "android.app.Instrumentation"
    }

    buildTypes {
        debug {
            applicationIdSuffix = ".debug"
            versionNameSuffix = "-debug"
            isDebuggable = true
        }
        release {
            isMinifyEnabled = true
            isShrinkResources = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro",
            )
        }
    }

    buildFeatures {
        buildConfig = true
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    lint {
        abortOnError = true
        checkDependencies = true
        warningsAsErrors = true
        // Keep the validated/published toolchain pinned; do not suppress product diagnostics.
        disable += setOf("AndroidGradlePluginVersion", "GradleDependency", "OldTargetApi")
    }
}

dependencies {
    testImplementation("junit:junit:4.13.2")
}
