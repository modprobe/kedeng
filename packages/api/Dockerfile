FROM gradle:9-jdk24-corretto AS cache

RUN mkdir /home/gradle/cache_home
ENV GRADLE_USER_HOME=/home/gradle/cache_home

COPY build.gradle.kts settings.gradle.kts gradle.properties /home/gradle/app/
COPY gradle/libs.versions.toml /home/gradle/app/gradle/
COPY server/build.gradle.kts /home/gradle/app/server/
WORKDIR /home/gradle/app

RUN gradle dependencies --no-daemon

FROM gradle:9-jdk24-corretto AS builder

COPY --from=cache /home/gradle/cache_home /home/gradle/.gradle
COPY --chown=gradle:gradle . /home/gradle/src

WORKDIR /home/gradle/src

RUN gradle buildFatJar --no-daemon

ARG OTEL_JAVAAGENT_VERSION="v2.21.0"
RUN wget -O /home/gradle/src/server/build/libs/otel-javaagent.jar \
    https://github.com/open-telemetry/opentelemetry-java-instrumentation/releases/download/${OTEL_JAVAAGENT_VERSION}/opentelemetry-javaagent.jar

FROM amazoncorretto:24 AS runtime

EXPOSE 8080
RUN mkdir /app
COPY --from=builder /home/gradle/src/server/build/libs/server-all.jar /app/api.jar
COPY --from=builder /home/gradle/src/server/build/libs/otel-javaagent.jar /app/otel-javaagent.jar

ENTRYPOINT ["java", "-javaagent:/app/otel-javaagent.jar" ,"-jar", "/app/api.jar"]