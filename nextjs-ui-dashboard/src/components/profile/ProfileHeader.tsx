/**
 * ProfileHeader Component
 *
 * User profile header with avatar upload, display name, and member info.
 */

import { useState } from 'react';
import { GlassCard } from '@/components/ui/GlassCard';
import { Avatar, AvatarImage, AvatarFallback } from '@/components/ui/avatar';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Camera, Edit2, Check, X, BadgeCheck } from 'lucide-react';

interface ProfileHeaderProps {
  user?: {
    name: string;
    email: string;
    avatarUrl?: string;
    memberSince: Date;
    verified?: boolean;
  };
}

export function ProfileHeader({ user }: ProfileHeaderProps) {
  const [isEditing, setIsEditing] = useState(false);
  const [displayName, setDisplayName] = useState(user?.name || 'User');
  const [tempName, setTempName] = useState(displayName);

  const handleSave = () => {
    setDisplayName(tempName);
    setIsEditing(false);
    // TODO: Save to API
  };

  const handleCancel = () => {
    setTempName(displayName);
    setIsEditing(false);
  };

  const handleAvatarUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) {
      // TODO: Upload to server
      const reader = new FileReader();
      reader.onloadend = () => {
        // TODO: Implement avatar upload to server
        // For now, preview locally (actual upload will be handled by API)
      };
      reader.readAsDataURL(file);
    }
  };

  const initials = displayName
    .split(' ')
    .map((n) => n[0])
    .join('')
    .toUpperCase()
    .substring(0, 2);

  return (
    <GlassCard className="relative overflow-hidden">
      {/* Background gradient */}
      <div className="absolute inset-0 bg-gradient-to-br from-sky-500/10 to-purple-500/10" />

      <div className="relative flex flex-col md:flex-row items-center md:items-start gap-6 p-6">
        {/* Avatar */}
        <div className="relative group">
          <Avatar className="w-24 h-24 md:w-32 md:h-32 border-4 border-slate-800">
            <AvatarImage src={user?.avatarUrl} alt={displayName} />
            <AvatarFallback className="bg-gradient-to-br from-sky-500 to-purple-500 text-white text-2xl font-bold">
              {initials}
            </AvatarFallback>
          </Avatar>

          {/* Upload overlay */}
          <label
            htmlFor="avatar-upload"
            className="absolute inset-0 flex items-center justify-center bg-black/60 rounded-full opacity-0 group-hover:opacity-100 transition-opacity cursor-pointer"
          >
            <Camera className="w-8 h-8 text-white" />
            <input
              id="avatar-upload"
              type="file"
              accept="image/*"
              className="hidden"
              onChange={handleAvatarUpload}
            />
          </label>
        </div>

        {/* User Info */}
        <div className="flex-1 text-center md:text-left">
          <div className="flex items-center justify-center md:justify-start gap-2 mb-2">
            {isEditing ? (
              <div className="flex items-center gap-2">
                <Input
                  value={tempName}
                  onChange={(e) => setTempName(e.target.value)}
                  className="max-w-xs bg-slate-800 border-slate-700"
                />
                <Button size="sm" onClick={handleSave} className="bg-green-600 hover:bg-green-700">
                  <Check className="w-4 h-4" />
                </Button>
                <Button
                  size="sm"
                  variant="outline"
                  onClick={handleCancel}
                  className="border-slate-700"
                >
                  <X className="w-4 h-4" />
                </Button>
              </div>
            ) : (
              <>
                <h1 className="text-3xl font-bold text-gray-100">{displayName}</h1>
                {user?.verified && (
                  <BadgeCheck className="w-6 h-6 text-sky-500" title="Verified account" />
                )}
                <Button
                  size="sm"
                  variant="ghost"
                  onClick={() => setIsEditing(true)}
                  className="text-gray-400 hover:text-gray-100"
                >
                  <Edit2 className="w-4 h-4" />
                </Button>
              </>
            )}
          </div>

          <p className="text-gray-400 mb-4">{user?.email}</p>

          <div className="flex flex-wrap items-center justify-center md:justify-start gap-4 text-sm text-gray-400">
            <div>
              <span className="text-gray-500">Member since</span>{' '}
              <span className="text-gray-300">
                {user?.memberSince.toLocaleDateString('en-US', {
                  month: 'long',
                  year: 'numeric',
                })}
              </span>
            </div>
          </div>
        </div>
      </div>
    </GlassCard>
  );
}
