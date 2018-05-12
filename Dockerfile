FROM jrottenberg/ffmpeg:3.4-centos
LABEL maintainer="code@waisbrot.net" \
      name="reencode" \
      vendor="code@waisbrot.net"
WORKDIR /opt/reencode
RUN yum install -q -y epel-release
RUN yum install -q -y perl-Image-ExifTool
RUN yum install -q -y python34 python34-pip
ENTRYPOINT ["python3", "-m", "reencode"]
ADD requirements.txt .
RUN pip3 install -r requirements.txt
ADD . .
EXPOSE 8081
