terraform {
  required_version = ">= 1.5.0"
  
  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 5.0"
    }
    google = {
      source  = "hashicorp/google"
      version = "~> 5.0"
    }
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "~> 2.23"
    }
    helm = {
      source  = "hashicorp/helm"
      version = "~> 2.11"
    }
  }
  
  backend "s3" {
    bucket         = "bot-core-terraform-state"
    key            = "global/terraform.tfstate"
    region         = "us-east-1"
    encrypt        = true
    dynamodb_table = "terraform-state-lock"
  }
}

# Multi-region provider configuration
provider "aws" {
  alias  = "us_east_1"
  region = "us-east-1"
}

provider "aws" {
  alias  = "eu_west_1"
  region = "eu-west-1"
}

provider "aws" {
  alias  = "ap_southeast_1"
  region = "ap-southeast-1"
}

provider "google" {
  project = var.gcp_project_id
  region  = "asia-southeast1"
}

# Global variables
variable "environment" {
  description = "Environment name"
  type        = string
  default     = "production"
}

variable "gcp_project_id" {
  description = "GCP Project ID"
  type        = string
}

# Global Route53 Hosted Zone
resource "aws_route53_zone" "main" {
  name = "bot-core.com"
  
  tags = {
    Environment = var.environment
    Purpose     = "Global DNS"
  }
}

# Global CloudFront Distribution
resource "aws_cloudfront_distribution" "global_cdn" {
  enabled             = true
  is_ipv6_enabled     = true
  comment             = "Bot Core Global CDN"
  default_root_object = "index.html"
  
  origin {
    domain_name = module.us_east_1.alb_dns_name
    origin_id   = "us-east-1-alb"
    
    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }
  
  origin {
    domain_name = module.eu_west_1.alb_dns_name
    origin_id   = "eu-west-1-alb"
    
    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }
  
  origin {
    domain_name = module.gcp_asia.lb_ip_address
    origin_id   = "gcp-asia-lb"
    
    custom_origin_config {
      http_port              = 80
      https_port             = 443
      origin_protocol_policy = "https-only"
      origin_ssl_protocols   = ["TLSv1.2"]
    }
  }
  
  default_cache_behavior {
    allowed_methods  = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods   = ["GET", "HEAD", "OPTIONS"]
    target_origin_id = "us-east-1-alb"
    
    forwarded_values {
      query_string = true
      headers      = ["*"]
      
      cookies {
        forward = "all"
      }
    }
    
    viewer_protocol_policy = "redirect-to-https"
    min_ttl                = 0
    default_ttl            = 86400
    max_ttl                = 31536000
    compress               = true
  }
  
  ordered_cache_behavior {
    path_pattern     = "/api/*"
    allowed_methods  = ["DELETE", "GET", "HEAD", "OPTIONS", "PATCH", "POST", "PUT"]
    cached_methods   = ["GET", "HEAD"]
    target_origin_id = "us-east-1-alb"
    
    forwarded_values {
      query_string = true
      headers      = ["Authorization", "Content-Type"]
      
      cookies {
        forward = "all"
      }
    }
    
    viewer_protocol_policy = "https-only"
    min_ttl                = 0
    default_ttl            = 0
    max_ttl                = 0
  }
  
  origin_group {
    origin_id = "multi-region-group"
    
    failover_criteria {
      status_codes = [500, 502, 503, 504]
    }
    
    member {
      origin_id = "us-east-1-alb"
    }
    
    member {
      origin_id = "eu-west-1-alb"
    }
    
    member {
      origin_id = "gcp-asia-lb"
    }
  }
  
  restrictions {
    geo_restriction {
      restriction_type = "none"
    }
  }
  
  viewer_certificate {
    acm_certificate_arn = aws_acm_certificate_validation.main.certificate_arn
    ssl_support_method  = "sni-only"
  }
  
  custom_error_response {
    error_code         = 404
    response_code      = 200
    response_page_path = "/index.html"
  }
  
  price_class = "PriceClass_All"
  
  tags = {
    Environment = var.environment
  }
}

# ACM Certificate for CloudFront
resource "aws_acm_certificate" "main" {
  provider          = aws.us_east_1
  domain_name       = "bot-core.com"
  validation_method = "DNS"
  
  subject_alternative_names = [
    "*.bot-core.com",
    "www.bot-core.com"
  ]
  
  lifecycle {
    create_before_destroy = true
  }
}

resource "aws_acm_certificate_validation" "main" {
  provider                = aws.us_east_1
  certificate_arn         = aws_acm_certificate.main.arn
  validation_record_fqdns = [for record in aws_route53_record.cert_validation : record.fqdn]
}

resource "aws_route53_record" "cert_validation" {
  for_each = {
    for dvo in aws_acm_certificate.main.domain_validation_options : dvo.domain_name => {
      name   = dvo.resource_record_name
      record = dvo.resource_record_value
      type   = dvo.resource_record_type
    }
  }
  
  allow_overwrite = true
  name            = each.value.name
  records         = [each.value.record]
  ttl             = 60
  type            = each.value.type
  zone_id         = aws_route53_zone.main.zone_id
}

# Route53 Health Checks
resource "aws_route53_health_check" "us_east_1" {
  fqdn              = module.us_east_1.alb_dns_name
  port              = 443
  type              = "HTTPS"
  resource_path     = "/health"
  failure_threshold = "3"
  request_interval  = "30"
  
  tags = {
    Name = "us-east-1-health-check"
  }
}

resource "aws_route53_health_check" "eu_west_1" {
  fqdn              = module.eu_west_1.alb_dns_name
  port              = 443
  type              = "HTTPS"
  resource_path     = "/health"
  failure_threshold = "3"
  request_interval  = "30"
  
  tags = {
    Name = "eu-west-1-health-check"
  }
}

# Route53 Records with Failover
resource "aws_route53_record" "www" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "www.bot-core.com"
  type    = "A"
  
  alias {
    name                   = aws_cloudfront_distribution.global_cdn.domain_name
    zone_id                = aws_cloudfront_distribution.global_cdn.hosted_zone_id
    evaluate_target_health = false
  }
}

resource "aws_route53_record" "api_primary" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "api.bot-core.com"
  type    = "A"
  ttl     = 60
  
  weighted_routing_policy {
    weight = 70
  }
  
  set_identifier  = "us-east-1"
  records         = [module.us_east_1.alb_dns_name]
  health_check_id = aws_route53_health_check.us_east_1.id
}

resource "aws_route53_record" "api_secondary" {
  zone_id = aws_route53_zone.main.zone_id
  name    = "api.bot-core.com"
  type    = "A"
  ttl     = 60
  
  weighted_routing_policy {
    weight = 30
  }
  
  set_identifier  = "eu-west-1"
  records         = [module.eu_west_1.alb_dns_name]
  health_check_id = aws_route53_health_check.eu_west_1.id
}

# Regional Deployments
module "us_east_1" {
  source = "./modules/regional-deployment"
  
  providers = {
    aws = aws.us_east_1
  }
  
  region           = "us-east-1"
  environment      = var.environment
  vpc_cidr         = "10.0.0.0/16"
  cluster_name     = "bot-core-us-east-1"
  instance_types   = ["t3.large", "t3.xlarge"]
  min_size         = 3
  max_size         = 10
  desired_capacity = 5
  
  database_config = {
    engine            = "aurora-mysql"
    instance_class    = "db.r5.large"
    allocated_storage = 100
    multi_az          = true
    backup_retention  = 30
  }
  
  redis_config = {
    node_type       = "cache.r6g.large"
    num_cache_nodes = 3
    engine_version  = "7.0"
  }
  
  monitoring_enabled = true
  backup_enabled     = true
}

module "eu_west_1" {
  source = "./modules/regional-deployment"
  
  providers = {
    aws = aws.eu_west_1
  }
  
  region           = "eu-west-1"
  environment      = var.environment
  vpc_cidr         = "10.1.0.0/16"
  cluster_name     = "bot-core-eu-west-1"
  instance_types   = ["t3.large", "t3.xlarge"]
  min_size         = 2
  max_size         = 8
  desired_capacity = 3
  
  database_config = {
    engine            = "aurora-mysql"
    instance_class    = "db.r5.large"
    allocated_storage = 100
    multi_az          = true
    backup_retention  = 30
  }
  
  redis_config = {
    node_type       = "cache.r6g.large"
    num_cache_nodes = 2
    engine_version  = "7.0"
  }
  
  monitoring_enabled = true
  backup_enabled     = true
}

module "gcp_asia" {
  source = "./modules/gcp-regional-deployment"
  
  project_id       = var.gcp_project_id
  region           = "asia-southeast1"
  environment      = var.environment
  network_name     = "bot-core-asia"
  subnet_cidr      = "10.2.0.0/16"
  cluster_name     = "bot-core-asia"
  
  node_pools = {
    general = {
      machine_type = "n2-standard-4"
      min_count    = 2
      max_count    = 6
      disk_size_gb = 100
      disk_type    = "pd-ssd"
    }
  }
  
  database_config = {
    tier              = "db-n1-standard-4"
    disk_size         = 100
    disk_type         = "PD_SSD"
    backup_enabled    = true
    high_availability = true
  }
  
  redis_config = {
    tier           = "STANDARD_HA"
    memory_size_gb = 5
    replica_count  = 2
  }
}

# Global Database Sync
resource "aws_dms_replication_instance" "main" {
  replication_instance_class = "dms.c5.large"
  replication_instance_id    = "bot-core-replication"
  
  allocated_storage            = 100
  apply_immediately            = true
  auto_minor_version_upgrade   = false
  engine_version               = "3.4.7"
  multi_az                     = true
  publicly_accessible          = false
  replication_subnet_group_id  = aws_dms_replication_subnet_group.main.id
  vpc_security_group_ids       = [aws_security_group.dms.id]
  
  tags = {
    Name        = "bot-core-dms"
    Environment = var.environment
  }
}

# Outputs
output "cloudfront_domain" {
  value = aws_cloudfront_distribution.global_cdn.domain_name
}

output "regional_endpoints" {
  value = {
    us_east_1 = module.us_east_1.endpoint
    eu_west_1 = module.eu_west_1.endpoint
    gcp_asia  = module.gcp_asia.endpoint
  }
}

output "health_check_ids" {
  value = {
    us_east_1 = aws_route53_health_check.us_east_1.id
    eu_west_1 = aws_route53_health_check.eu_west_1.id
  }
}