terraform {
  required_version = ">= 1.9.0"
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.73"
    }
  }
}

variable "region" {
  type = string
}

variable "name_prefix" {
  type = string
}

variable "resource_group_key" {
  type = string
}

variable "resource_group_value" {
  type = string
}

variable "elastic_ip_id" {
  type = string
}

variable "instance_type" {
  type = string
}

variable "ubuntu_version" {
  type = string
}

variable "volume_size_gb" {
  type = number
}

variable "component" {
  type = string
}

provider "aws" {
  region = var.region
  default_tags {
    tags = {
      (var.resource_group_key) = var.resource_group_value
    }
  }
}

resource "aws_resourcegroups_group" "main" {
  name = "${var.name_prefix}_resource_group"
  resource_query {
    query = jsonencode({
      ResourceTypeFilters = ["AWS::AllSupported"],
      TagFilters = [{
        Key    = var.resource_group_key,
        Values = [var.resource_group_value]
      }]
    })
  }
}

resource "tls_private_key" "main" {
  algorithm = "RSA"
  rsa_bits  = 4096
}

resource "local_file" "ssh_key_pk_txt" {
  filename        = "ssh_key_pk.txt"
  file_permission = "0600"
  content         = tls_private_key.main.public_key_openssh
}

resource "local_file" "ssh_key_sk_txt" {
  filename        = "ssh_key_sk.txt"
  file_permission = "0600"
  content         = tls_private_key.main.private_key_openssh
}

resource "aws_key_pair" "main" {
  key_name   = "${var.name_prefix}_ssh_key"
  public_key = tls_private_key.main.public_key_openssh
}

resource "aws_security_group" "main" {
  tags = {
    Name = "${var.name_prefix}_sg"
  }
  name        = "${var.name_prefix}_sg"
  description = "${var.name_prefix}_sg"
}

resource "aws_vpc_security_group_egress_rule" "main_tx_self" {
  tags = {
    Name = "${var.name_prefix}_sg_tx_self"
  }
  description                  = "${var.name_prefix}_sg_tx_self"
  security_group_id            = aws_security_group.main.id
  referenced_security_group_id = aws_security_group.main.id
  ip_protocol                  = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "main_rx_self" {
  tags = {
    Name = "${var.name_prefix}_sg_rx_self"
  }
  description                  = "${var.name_prefix}_sg_rx_self"
  security_group_id            = aws_security_group.main.id
  referenced_security_group_id = aws_security_group.main.id
  ip_protocol                  = "-1"
}

resource "aws_vpc_security_group_egress_rule" "main_tx_any" {
  tags = {
    Name = "${var.name_prefix}_sg_tx_any"
  }
  description       = "${var.name_prefix}_sg_tx_any"
  security_group_id = aws_security_group.main.id
  cidr_ipv4         = "0.0.0.0/0"
  ip_protocol       = "-1"
}

resource "aws_vpc_security_group_ingress_rule" "main_rx_any_tcp_22" {
  tags = {
    Name = "${var.name_prefix}_sg_rx_any_tcp_22"
  }
  description       = "${var.name_prefix}_sg_rx_any_tcp_22"
  security_group_id = aws_security_group.main.id
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 22
  to_port           = 22
  ip_protocol       = "tcp"
}

resource "aws_vpc_security_group_ingress_rule" "main_rx_any_tcp_80" {
  tags = {
    Name = "${var.name_prefix}_sg_rx_any_tcp_80"
  }
  description       = "${var.name_prefix}_sg_rx_any_tcp_80"
  security_group_id = aws_security_group.main.id
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 80
  to_port           = 80
  ip_protocol       = "tcp"
}

resource "aws_vpc_security_group_ingress_rule" "main_rx_any_tcp_443" {
  tags = {
    Name = "${var.name_prefix}_sg_rx_any_tcp_443"
  }
  description       = "${var.name_prefix}_sg_rx_any_tcp_443"
  security_group_id = aws_security_group.main.id
  cidr_ipv4         = "0.0.0.0/0"
  from_port         = 443
  to_port           = 443
  ip_protocol       = "tcp"
}

data "aws_ami" "main" {
  owners = ["099720109477"]
  filter {
    name   = "name"
    values = ["ubuntu/images/*-${var.ubuntu_version}-amd64-server-*"]
  }
  most_recent = true
}

resource "aws_instance" "main" {
  tags = {
    Name = "${var.name_prefix}_instance"
  }
  ami                    = data.aws_ami.main.id
  instance_type          = var.instance_type
  key_name               = aws_key_pair.main.key_name
  vpc_security_group_ids = [aws_security_group.main.id]
  root_block_device {
    volume_size = var.volume_size_gb
  }
}

data "aws_eip" "main" {
  id = var.elastic_ip_id
}

resource "aws_eip_association" "main" {
  allocation_id       = data.aws_eip.main.id
  instance_id         = aws_instance.main.id
  allow_reassociation = false
}

resource "terraform_data" "provisioner" {
  depends_on = [
    aws_eip_association.main,
  ]
  connection {
    type        = "ssh"
    agent       = false
    user        = "ubuntu"
    host        = data.aws_eip.main.public_ip
    private_key = tls_private_key.main.private_key_openssh
  }
  provisioner "file" {
    source      = "./${var.component}/"
    destination = "/home/ubuntu"
  }
  provisioner "file" {
    source      = "./terraform/provision.bash"
    destination = "/home/ubuntu/provision.bash"
  }
  provisioner "remote-exec" {
    inline = [
      "cd /home/ubuntu || exit $?",
      "bash provision.bash || exit $?",
    ]
  }
}

resource "local_file" "instance_id_txt" {
  depends_on = [
    terraform_data.provisioner,
  ]
  filename        = "instance_id.txt"
  file_permission = "0600"
  content         = "${aws_instance.main.id}\n"
}
